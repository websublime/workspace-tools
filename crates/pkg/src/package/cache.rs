use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    data: T,
    timestamp: Instant,
}

impl<T: Clone> CacheEntry<T> {
    pub fn new(data: T) -> Self {
        Self { data, timestamp: Instant::now() }
    }

    pub fn is_valid(&self, ttl: Duration) -> bool {
        self.timestamp.elapsed() < ttl
    }

    pub fn get(&self) -> T {
        self.data.clone()
    }
}
