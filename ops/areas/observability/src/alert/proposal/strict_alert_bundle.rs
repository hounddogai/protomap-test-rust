use super::*;

use acme_kit::text;

/// Where a strict alert bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictAlertBundleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl StrictAlertBundleStatus {
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

impl std::fmt::Display for StrictAlertBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict alert bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictAlertBundle {
    pub code: String,
    pub name: String,
    pub starts_at: i64,
    pub approved: bool,
    pub attempts: i64,
    pub cipher: String,
    pub batch_size: i64,
    pub audience: String,
    pub status: StrictAlertBundleStatus,
}

impl StrictAlertBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictAlertBundleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictAlertBundleStatus) -> bool {
        match self.status {
            StrictAlertBundleStatus::Planned => {
                return next == StrictAlertBundleStatus::Completed;
            }
            StrictAlertBundleStatus::Scheduled => {
                match next {
                    StrictAlertBundleStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictAlertBundleStatus::InProgress => {
                return next == StrictAlertBundleStatus::Completed || next == StrictAlertBundleStatus::Cancelled;
            }
            StrictAlertBundleStatus::Completed => {
                return next == StrictAlertBundleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictAlertBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict alert bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.batch_size as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictAlertBundle {
        StrictAlertBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            starts_at: self.starts_at,
            approved: self.approved,
            attempts: self.attempts,
            cipher: self.cipher.clone(),
            batch_size: self.batch_size,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a strict alert bundle in status planned.
pub fn new_strict_alert_bundle(code: &str, name: &str) -> StrictAlertBundle {
    StrictAlertBundle {
        code: code.to_string(),
        name: name.to_string(),
        starts_at: 0,
        approved: false,
        attempts: 0,
        cipher: "".to_string(),
        batch_size: 0,
        audience: "".to_string(),
        status: StrictAlertBundleStatus::Planned,
    }
}
