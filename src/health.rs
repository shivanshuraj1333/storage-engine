use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct HealthCheck {
    is_healthy: AtomicBool,
    last_successful_write: AtomicU64,
    message_queue_size: AtomicU64,
    total_messages_processed: AtomicU64,
    failed_writes: AtomicU64,
}

impl HealthCheck {
    pub fn new() -> Self {
        Self {
            is_healthy: AtomicBool::new(true),
            last_successful_write: AtomicU64::new(0),
            message_queue_size: AtomicU64::new(0),
            total_messages_processed: AtomicU64::new(0),
            failed_writes: AtomicU64::new(0),
        }
    }

    pub fn update_status(&self, healthy: bool) {
        self.is_healthy.store(healthy, Ordering::SeqCst);
    }

    pub fn record_successful_write(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.last_successful_write.store(now, Ordering::SeqCst);
        self.total_messages_processed.fetch_add(1, Ordering::SeqCst);
    }

    pub fn record_failed_write(&self) {
        self.failed_writes.fetch_add(1, Ordering::SeqCst);
        // If too many failures, mark as unhealthy
        if self.failed_writes.load(Ordering::SeqCst) > 5 {
            self.update_status(false);
        }
    }

    pub fn update_queue_size(&self, size: u64) {
        self.message_queue_size.store(size, Ordering::SeqCst);
    }

    pub fn get_health_status(&self) -> HealthStatus {
        HealthStatus {
            is_healthy: self.is_healthy.load(Ordering::SeqCst),
            last_write: self.last_successful_write.load(Ordering::SeqCst),
            queue_size: self.message_queue_size.load(Ordering::SeqCst),
            total_processed: self.total_messages_processed.load(Ordering::SeqCst),
            failed_writes: self.failed_writes.load(Ordering::SeqCst),
        }
    }
}

#[derive(Debug)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub last_write: u64,
    pub queue_size: u64,
    pub total_processed: u64,
    pub failed_writes: u64,
} 