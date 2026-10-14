use super::*;

use acme_kit::text;

/// Where a batched postmortem target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedPostmortemTargetStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BatchedPostmortemTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BatchedPostmortemTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched postmortem target as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPostmortemTarget {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub rotation_days: i64,
    pub external_ref: String,
    pub priority: i64,
    pub expires_at: i64,
    pub reviewer: String,
    pub status: BatchedPostmortemTargetStatus,
}

impl BatchedPostmortemTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedPostmortemTargetStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedPostmortemTargetStatus) -> bool {
        match self.status {
            BatchedPostmortemTargetStatus::Requested => {
                match next {
                    BatchedPostmortemTargetStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPostmortemTargetStatus::Granted => {
                return next == BatchedPostmortemTargetStatus::Revoked
                    || next == BatchedPostmortemTargetStatus::Expired;
            }
            BatchedPostmortemTargetStatus::Revoked => {
                match next {
                    BatchedPostmortemTargetStatus::Expired => {
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

    pub fn move_to(&mut self, next: BatchedPostmortemTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched postmortem target {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.expires_at as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedPostmortemTarget {
        BatchedPostmortemTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            phone: self.phone.clone(),
            rotation_days: self.rotation_days,
            external_ref: self.external_ref.clone(),
            priority: self.priority,
            expires_at: self.expires_at,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched postmortem target in status requested.
pub fn new_batched_postmortem_target(code: &str, label: &str) -> BatchedPostmortemTarget {
    BatchedPostmortemTarget {
        code: code.to_string(),
        label: label.to_string(),
        phone: "".to_string(),
        rotation_days: 0,
        external_ref: "".to_string(),
        priority: 0,
        expires_at: 0,
        reviewer: "".to_string(),
        status: BatchedPostmortemTargetStatus::Requested,
    }
}
