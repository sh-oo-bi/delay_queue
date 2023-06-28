use tokio_util::time::{delay_queue, DelayQueue};

use futures::ready;
use std::collections::HashMap;
use std::task::{Context, Poll};
use std::time::Duration;

type CacheKey = String;
type Value = String;

struct Cache {
    entries: HashMap<CacheKey, (Value, delay_queue::Key)>,
    expirations: DelayQueue<CacheKey>,
}

const TTL_SECS: u64 = 10;

impl Cache {
    fn new() -> Cache {
        Cache {
            entries: HashMap::new(),
            expirations: DelayQueue::new(),
        }
    }

    fn insert(&mut self, key: CacheKey, value: Value) {
        let delay = self
            .expirations
            .insert(key.clone(), Duration::from_secs(TTL_SECS));

        self.entries.insert(key, (value, delay));
    }

    fn _get(&self, key: &CacheKey) -> Option<&Value> {
        self.entries.get(key).map(|&(ref v, _)| v)
    }

    fn _remove(&mut self, key: &CacheKey) {
        if let Some((_, cache_key)) = self.entries.remove(key) {
            self.expirations.remove(&cache_key);
        }
    }

    fn poll_purge(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        while let Some(entry) = ready!(self.expirations.poll_expired(cx)) {
            println!("{}",entry.get_ref());
            self.entries.remove(entry.get_ref());
        }

        Poll::Ready(())
    }

    async fn purge(&mut self) {
        futures::future::poll_fn(|cx| self.poll_purge(cx)).await;
    }
}

#[tokio::main]
async fn main() {
    let mut cache = Cache::new();
    cache.insert("k1".to_string(), "v1".to_string());
    cache.insert("k2".to_string(), "v2".to_string());
    cache.insert("k3".to_string(), "v3".to_string());
    cache.insert("k4".to_string(), "v4".to_string());
    cache.insert("k5".to_string(), "v5".to_string());

    cache.purge().await;
}
