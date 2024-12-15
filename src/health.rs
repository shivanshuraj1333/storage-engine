use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::warn;
use serde::Serialize;

/// Component for monitoring and reporting system health metrics.
/// Uses atomic types for thread-safe access to health indicators.
pub struct HealthCheck {
    /// Overall health status of the system
    is_healthy: AtomicBool,
    /// Timestamp of the last successful write operation
    last_successful_write: AtomicU64,
    /// Current size of the message processing queue
    message_queue_size: AtomicU64,
    /// Total number of messages processed since startup
    total_messages_processed: AtomicU64,
    /// Number of failed write operations
    failed_writes: AtomicU64,
}

impl HealthCheck {
    /// Creates a new HealthCheck instance with default values
    pub fn new() -> Self {
        Self {
            is_healthy: AtomicBool::new(true),
            last_successful_write: AtomicU64::new(0),
            message_queue_size: AtomicU64::new(0),
            total_messages_processed: AtomicU64::new(0),
            failed_writes: AtomicU64::new(0),
        }
    }

    /// Updates the overall health status
    pub fn update_status(&self, healthy: bool) {
        self.is_healthy.store(healthy, Ordering::SeqCst);
    }

    /// Records a successful write operation
    pub fn record_successful_write(&self) {
        // Update last successful write timestamp
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|e| {
                warn!("Failed to get system time: {}", e);
                std::time::Duration::from_secs(0)
            })
            .as_secs();
        self.last_successful_write.store(now, Ordering::SeqCst);

        // Increment total processed count
        self.total_messages_processed.fetch_add(1, Ordering::SeqCst);

        // Reset failed writes if any successful write occurs
        if self.failed_writes.load(Ordering::SeqCst) > 0 {
            self.failed_writes.store(0, Ordering::SeqCst);
            self.update_status(true);
        }
    }

    /// Records a failed write operation
    pub fn record_failed_write(&self) {
        let failed = self.failed_writes.fetch_add(1, Ordering::SeqCst) + 1;
        
        // Mark system as unhealthy if too many failures
        if failed > 5 {
            warn!("System marked unhealthy due to {} consecutive write failures", failed);
            self.update_status(false);
        }
    }

    /// Updates the current message queue size
    pub fn update_queue_size(&self, size: u64) {
        self.message_queue_size.store(size, Ordering::SeqCst);
    }

    /// Returns the current health status and metrics
    pub fn get_health_status(&self) -> HealthStatus {
        HealthStatus {
            is_healthy: self.is_healthy.load(Ordering::SeqCst),
            last_write: self.last_successful_write.load(Ordering::SeqCst),
            queue_size: self.message_queue_size.load(Ordering::SeqCst),
            total_processed: self.total_messages_processed.load(Ordering::SeqCst),
            failed_writes: self.failed_writes.load(Ordering::SeqCst),
        }
    }

    /// Returns a detailed health report
    pub fn get_detailed_status(&self) -> DetailedHealthStatus {
        DetailedHealthStatus {
            is_healthy: self.is_healthy.load(Ordering::SeqCst),
            last_write: self.last_successful_write.load(Ordering::SeqCst),
            queue_size: self.message_queue_size.load(Ordering::SeqCst),
            total_processed: self.total_messages_processed.load(Ordering::SeqCst),
            failed_writes: self.failed_writes.load(Ordering::SeqCst),
            uptime_seconds: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

/// Represents the current health status and metrics of the system
#[derive(Debug, Serialize)]
pub struct HealthStatus {
    /// Whether the system is currently healthy
    pub is_healthy: bool,
    /// Timestamp of the last successful write
    pub last_write: u64,
    /// Current size of the message queue
    pub queue_size: u64,
    /// Total number of messages processed
    pub total_processed: u64,
    /// Number of consecutive write failures
    pub failed_writes: u64,
}

#[derive(Debug, Serialize)]
pub struct DetailedHealthStatus {
    pub is_healthy: bool,
    pub last_write: u64,
    pub queue_size: u64,
    pub total_processed: u64,
    pub failed_writes: u64,
    pub uptime_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_successful_write() {
        let health = HealthCheck::new();
        health.record_successful_write();
        
        let status = health.get_health_status();
        assert!(status.is_healthy);
        assert_eq!(status.total_processed, 1);
        assert_eq!(status.failed_writes, 0);
    }

    #[test]
    fn test_failed_writes() {
        let health = HealthCheck::new();
        
        // Record 6 failed writes
        for _ in 0..6 {
            health.record_failed_write();
        }
        
        let status = health.get_health_status();
        assert!(!status.is_healthy);
        assert_eq!(status.failed_writes, 6);
    }

    #[test]
    fn test_recovery() {
        let health = HealthCheck::new();
        
        // Record failures then a success
        for _ in 0..3 {
            health.record_failed_write();
        }
        health.record_successful_write();
        
        let status = health.get_health_status();
        assert!(status.is_healthy);
        assert_eq!(status.failed_writes, 0);
    }
} 