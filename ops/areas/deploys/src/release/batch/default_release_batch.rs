use super::*;

use acme_kit::text;

/// Where a default release batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultReleaseBatchStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DefaultReleaseBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for DefaultReleaseBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default release batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseBatch {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub pinned: bool,
    pub cluster: String,
    pub unit: String,
    pub duration_minutes: i64,
    pub expires_at: i64,
    pub maximum: f64,
    pub status: DefaultReleaseBatchStatus,
}

impl DefaultReleaseBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultReleaseBatchStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultReleaseBatchStatus) -> bool {
        match self.status {
            DefaultReleaseBatchStatus::Planned => {
                match next {
                    DefaultReleaseBatchStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultReleaseBatchStatus::Scheduled => {
                return next == DefaultReleaseBatchStatus::InProgress;
            }
            DefaultReleaseBatchStatus::InProgress => {
                match next {
                    DefaultReleaseBatchStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultReleaseBatchStatus::Completed => {
                match next {
                    DefaultReleaseBatchStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultReleaseBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default release batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        ((self.maximum - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultReleaseBatch {
        DefaultReleaseBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            pinned: self.pinned,
            cluster: self.cluster.clone(),
            unit: self.unit.clone(),
            duration_minutes: self.duration_minutes,
            expires_at: self.expires_at,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a default release batch in status planned.
pub fn new_default_release_batch(code: &str, title: &str) -> DefaultReleaseBatch {
    DefaultReleaseBatch {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        pinned: false,
        cluster: "".to_string(),
        unit: "".to_string(),
        duration_minutes: 0,
        expires_at: 0,
        maximum: 0.0,
        status: DefaultReleaseBatchStatus::Planned,
    }
}
