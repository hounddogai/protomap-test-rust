use super::*;

use acme_kit::text;

/// Where a batched overage bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedOverageBundleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BatchedOverageBundleStatus {
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

impl std::fmt::Display for BatchedOverageBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched overage bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedOverageBundle {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub phone: String,
    pub channel: String,
    pub margin: f64,
    pub host: String,
    pub status: BatchedOverageBundleStatus,
}

impl BatchedOverageBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedOverageBundleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedOverageBundleStatus) -> bool {
        match self.status {
            BatchedOverageBundleStatus::Planned => {
                match next {
                    BatchedOverageBundleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedOverageBundleStatus::Scheduled => {
                match next {
                    BatchedOverageBundleStatus::Completed | BatchedOverageBundleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedOverageBundleStatus::InProgress => {
                return next == BatchedOverageBundleStatus::Completed || next == BatchedOverageBundleStatus::Cancelled;
            }
            BatchedOverageBundleStatus::Completed => {
                return next == BatchedOverageBundleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedOverageBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched overage bundle {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "batched overage bundle", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.expires_at as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedOverageBundle {
        BatchedOverageBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            phone: self.phone.clone(),
            channel: self.channel.clone(),
            margin: self.margin,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a batched overage bundle in status planned.
pub fn new_batched_overage_bundle(code: &str, label: &str) -> BatchedOverageBundle {
    BatchedOverageBundle {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        phone: "".to_string(),
        channel: "".to_string(),
        margin: 0.0,
        host: "".to_string(),
        status: BatchedOverageBundleStatus::Planned,
    }
}
