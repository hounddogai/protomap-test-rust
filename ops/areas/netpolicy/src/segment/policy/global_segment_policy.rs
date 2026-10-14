use super::*;

use acme_kit::text;

/// Where a global segment policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalSegmentPolicyStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl GlobalSegmentPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for GlobalSegmentPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global segment policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSegmentPolicy {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub floor_value: f64,
    pub retries: i64,
    pub category: String,
    pub status: GlobalSegmentPolicyStatus,
}

impl GlobalSegmentPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalSegmentPolicyStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalSegmentPolicyStatus) -> bool {
        match self.status {
            GlobalSegmentPolicyStatus::Created => {
                match next {
                    GlobalSegmentPolicyStatus::Published | GlobalSegmentPolicyStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalSegmentPolicyStatus::Validated => {
                match next {
                    GlobalSegmentPolicyStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalSegmentPolicyStatus::Published => {
                match next {
                    GlobalSegmentPolicyStatus::Retired => {
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

    pub fn move_to(&mut self, next: GlobalSegmentPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global segment policy {} cannot move from {} to {}",
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

    pub fn adjusted_volume(&self) -> f64 {
        ((self.retries as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalSegmentPolicy {
        GlobalSegmentPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            floor_value: self.floor_value,
            retries: self.retries,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a global segment policy in status created.
pub fn new_global_segment_policy(code: &str, title: &str) -> GlobalSegmentPolicy {
    GlobalSegmentPolicy {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        floor_value: 0.0,
        retries: 0,
        category: "".to_string(),
        status: GlobalSegmentPolicyStatus::Created,
    }
}
