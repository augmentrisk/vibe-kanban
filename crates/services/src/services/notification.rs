use std::sync::Arc;

use tokio::sync::RwLock;

use crate::services::config::Config;

/// Service for handling notifications.
///
/// Desktop notifications (sound, OS push) have been removed as the application
/// is deployed as a headless SaaS service where they are non-functional.
#[derive(Debug, Clone)]
pub struct NotificationService {
    _config: Arc<RwLock<Config>>,
}

impl NotificationService {
    pub fn new(config: Arc<RwLock<Config>>) -> Self {
        Self { _config: config }
    }

    /// Notification handler — currently a no-op for SaaS deployment.
    pub async fn notify(&self, title: &str, message: &str) {
        tracing::debug!("Notification (no-op): {title} — {message}");
    }
}
