use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::sleep;

const DEFAULT_RATE_PER_SECOND: f64 = 5.0;
const MIN_RATE: f64 = 1.0;
const MAX_RATE: f64 = 20.0;
const BURST_CAPACITY: f64 = 10.0;

#[derive(Clone)]
pub struct RateLimiter {
    inner: Arc<Inner>,
}

struct Inner {
    rate: AtomicU64,
    tokens: RwLock<TokenBucket>,
}

#[derive(Clone)]
struct TokenBucket {
    tokens: f64,
    last_update: Instant,
}

impl RateLimiter {
    pub fn new(rate_per_second: f64) -> Self {
        let rate = rate_per_second.clamp(MIN_RATE, MAX_RATE);
        Self {
            inner: Arc::new(Inner {
                rate: AtomicU64::new((rate * 1000.0) as u64),
                tokens: RwLock::new(TokenBucket::new(BURST_CAPACITY)),
            }),
        }
    }

    pub fn default_rate() -> Self {
        Self::new(DEFAULT_RATE_PER_SECOND)
    }

    pub fn rate(&self) -> f64 {
        self.inner.rate.load(Ordering::Relaxed) as f64 / 1000.0
    }

    pub fn set_rate(&self, rate_per_second: f64) {
        let rate = rate_per_second.clamp(MIN_RATE, MAX_RATE);
        self.inner
            .rate
            .store((rate * 1000.0) as u64, Ordering::Relaxed);
    }

    pub async fn acquire(&self) {
        let (rate, tokens) = (self.inner.rate.load(Ordering::Relaxed), &self.inner.tokens);

        loop {
            let mut bucket = tokens.write().await;
            bucket.refill(rate);

            if bucket.tokens >= 1.0 {
                bucket.tokens -= 1.0;
                return;
            }

            let tokens_needed = 1.0 - bucket.tokens;
            let rate_per_ms = rate as f64;
            let wait_ms = (tokens_needed / rate_per_ms).ceil() as u64;
            drop(bucket);

            sleep(Duration::from_millis(wait_ms.max(1))).await;
        }
    }
}

impl TokenBucket {
    fn new(capacity: f64) -> Self {
        Self {
            tokens: capacity,
            last_update: Instant::now(),
        }
    }

    fn refill(&mut self, rate_millis: u64) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_millis() as f64;
        // rate_millis is already rate * 1000, so divide by 1000000 to get tokens/ms
        let rate_per_ms = rate_millis as f64 / 1_000_000.0;
        let new_tokens = elapsed * rate_per_ms;

        self.tokens = (self.tokens + new_tokens).min(BURST_CAPACITY);
        self.last_update = now;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_requests() {
        let limiter = RateLimiter::new(10.0);
        for _ in 0..5 {
            limiter.acquire().await;
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_respects_rate() {
        let limiter = RateLimiter::new(2.0);
        // Initial burst is 10, so first 10 requests should be instant
        for _ in 0..10 {
            limiter.acquire().await;
        }
        // 11th request should wait because we depleted the burst
        // At 2 req/sec, 1 token refills every 500ms
        let start = Instant::now();
        limiter.acquire().await;
        let elapsed = start.elapsed();
        // Should be at least 400ms (allowing for some timing variance)
        assert!(elapsed.as_millis() >= 400, "elapsed was {:?}", elapsed);
    }

    #[test]
    fn test_rate_limiter_clamping() {
        let limiter = RateLimiter::new(100.0);
        assert_eq!(limiter.rate(), MAX_RATE);

        let limiter = RateLimiter::new(0.5);
        assert_eq!(limiter.rate(), MIN_RATE);
    }
}
