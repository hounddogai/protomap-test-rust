use super::*;

use acme_kit::text;

/// Where a batched policy window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedPolicyWindowStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedPolicyWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for BatchedPolicyWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched policy window. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyWindow {
    pub code: String,
    pub title: String,
    pub locale: String,
    pub rotation_days: i64,
    pub port: i64,
    pub created_at: i64,
    pub batch_size: i64,
    pub status: BatchedPolicyWindowStatus,
}

impl BatchedPolicyWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedPolicyWindowStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedPolicyWindowStatus) -> bool {
        match self.status {
            BatchedPolicyWindowStatus::Draft => {
                return next == BatchedPolicyWindowStatus::Suspended;
            }
            BatchedPolicyWindowStatus::Pending => {
                match next {
                    BatchedPolicyWindowStatus::Active | BatchedPolicyWindowStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPolicyWindowStatus::Active => {
                match next {
                    BatchedPolicyWindowStatus::Suspended | BatchedPolicyWindowStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPolicyWindowStatus::Suspended => {
                match next {
                    BatchedPolicyWindowStatus::Closed => {
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

    pub fn move_to(&mut self, next: BatchedPolicyWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched policy window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.created_at as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedPolicyWindow {
        BatchedPolicyWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            locale: self.locale.clone(),
            rotation_days: self.rotation_days,
            port: self.port,
            created_at: self.created_at,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a batched policy window in status draft.
pub fn new_batched_policy_window(code: &str, title: &str) -> BatchedPolicyWindow {
    BatchedPolicyWindow {
        code: code.to_string(),
        title: title.to_string(),
        locale: "".to_string(),
        rotation_days: 0,
        port: 0,
        created_at: 0,
        batch_size: 0,
        status: BatchedPolicyWindowStatus::Draft,
    }
}
