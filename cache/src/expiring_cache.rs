use std::collections::HashMap;
use std::sync::Arc;

use std::time::{Duration, Instant};

use actix_web::http::{HeaderMap, StatusCode};
use actix_web::web::Bytes;
use blocking_delay_queue::{BlockingDelayQueue, DelayItem};
use crossbeam::sync::{ShardedLock, ShardedLockWriteGuard};

const INSERT_TIMEOUT: Duration = Duration::from_millis(30000);

#[derive(Clone)]
pub struct CachedResponse {
    pub status_code: StatusCode,
    pub headers: HeaderMap,
    pub body: Bytes,
    pub ttl: Instant,
}

pub struct ResponseCache {
    cache: ShardedLock<HashMap<Arc<str>, CachedResponse>>,
    expire_q: BlockingDelayQueue<DelayItem<Arc<str>>>,
    capacity: usize,
}

impl CachedResponse {
    pub fn expired(&self) -> bool {
        self.ttl < Instant::now()
    }
}

impl ResponseCache {
    pub fn with_capacity(capacity: usize) -> Self {
         ResponseCache {
            cache: ShardedLock::new(HashMap::new()),
            expire_q: BlockingDelayQueue::new_with_capacity(capacity),
            capacity,
        }
    }

    pub fn expire_head(&self) {
        let item = self.expire_q.take();
        self.cache_write_lock().remove(&item.data);
    }

    pub fn put(&self, k: Arc<str>, v: CachedResponse, ttl: Instant) -> bool {
        let mut cache = self.cache_write_lock();
        if cache.len() < self.capacity {
            // avoid blocking api, len should be same as map
            let success = self
                .expire_q
                .offer(DelayItem::new(k.clone(), ttl), INSERT_TIMEOUT);
            if success {
                cache.insert(k, v);
            }
            success
        } else {
            false
        }
    }

    pub fn get(&self, k: Arc<str>) -> Option<CachedResponse> {
        self.cache
            .read()
            .expect("Cache map stale!")
            .get(&k)
            .map_or_else(|| None, |v| Some(v.clone()))
    }

    fn cache_write_lock(&self) -> ShardedLockWriteGuard<'_, HashMap<Arc<str>, CachedResponse>> {
        self.cache.write().expect("Cache write stale!")
    }
}
