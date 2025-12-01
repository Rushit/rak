//! Invocation tracking for cancellation support

use dashmap::DashMap;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::ws_types::InvocationStatus;

/// Tracks active invocations and provides cancellation support
#[derive(Clone)]
pub struct InvocationTracker {
    /// Map of invocation_id to cancellation token and status
    active: DashMap<String, InvocationEntry>,
}

#[derive(Clone)]
struct InvocationEntry {
    token: CancellationToken,
    status: InvocationStatus,
}

impl InvocationTracker {
    /// Create a new invocation tracker
    pub fn new() -> Self {
        Self {
            active: DashMap::new(),
        }
    }

    /// Register a new invocation and return its ID and cancellation token
    pub fn register(&self) -> (String, CancellationToken) {
        let id = Uuid::new_v4().to_string();
        let token = CancellationToken::new();

        self.active.insert(
            id.clone(),
            InvocationEntry {
                token: token.clone(),
                status: InvocationStatus::Active,
            },
        );

        (id, token)
    }

    /// Cancel an invocation by its ID
    ///
    /// Returns true if the invocation was found and cancelled, false otherwise
    pub fn cancel(&self, invocation_id: &str) -> bool {
        if let Some(mut entry) = self.active.get_mut(invocation_id) {
            entry.token.cancel();
            entry.status = InvocationStatus::Cancelled;
            true
        } else {
            false
        }
    }

    /// Check if an invocation is cancelled
    pub fn is_cancelled(&self, invocation_id: &str) -> bool {
        self.active
            .get(invocation_id)
            .map(|entry| entry.token.is_cancelled())
            .unwrap_or(false)
    }

    /// Get the status of an invocation
    pub fn status(&self, invocation_id: &str) -> InvocationStatus {
        self.active
            .get(invocation_id)
            .map(|entry| entry.status.clone())
            .unwrap_or(InvocationStatus::NotFound)
    }

    /// Mark an invocation as completed and remove it from tracking
    pub fn complete(&self, invocation_id: &str) {
        if let Some(mut entry) = self.active.get_mut(invocation_id) {
            entry.status = InvocationStatus::Completed;
        }
        // Remove after a short delay to allow status queries
        // In production, this might use a TTL cache or background cleanup task
        self.active.remove(invocation_id);
    }

    /// Unregister an invocation (cleanup)
    pub fn unregister(&self, invocation_id: &str) {
        self.active.remove(invocation_id);
    }
}

impl Default for InvocationTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_cancel() {
        let tracker = InvocationTracker::new();
        let (id, token) = tracker.register();

        assert!(!token.is_cancelled());
        assert_eq!(tracker.status(&id), InvocationStatus::Active);

        assert!(tracker.cancel(&id));
        assert!(token.is_cancelled());
        assert_eq!(tracker.status(&id), InvocationStatus::Cancelled);
    }

    #[test]
    fn test_cancel_nonexistent() {
        let tracker = InvocationTracker::new();
        assert!(!tracker.cancel("nonexistent"));
    }

    #[test]
    fn test_status_not_found() {
        let tracker = InvocationTracker::new();
        assert!(matches!(
            tracker.status("nonexistent"),
            InvocationStatus::NotFound
        ));
    }

    #[test]
    fn test_complete() {
        let tracker = InvocationTracker::new();
        let (id, _token) = tracker.register();

        tracker.complete(&id);
        // After complete, status should be NotFound (removed)
        assert!(matches!(
            tracker.status(&id),
            InvocationStatus::NotFound
        ));
    }
}

