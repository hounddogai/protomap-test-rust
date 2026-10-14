use super::*;

use acme_kit::text;

/// Where a failover allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverAllocationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FailoverAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for FailoverAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One failover allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAllocation {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub revision: i64,
    pub expires_at: i64,
    pub region: String,
    pub status: FailoverAllocationStatus,
}

impl FailoverAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverAllocationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverAllocationStatus) -> bool {
        match self.status {
            FailoverAllocationStatus::Open => {
                return next == FailoverAllocationStatus::InReview || next == FailoverAllocationStatus::Archived;
            }
            FailoverAllocationStatus::InReview => {
                match next {
                    FailoverAllocationStatus::Resolved | FailoverAllocationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverAllocationStatus::Resolved => {
                match next {
                    FailoverAllocationStatus::Reopened | FailoverAllocationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverAllocationStatus::Reopened => {
                return next == FailoverAllocationStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.expires_at as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverAllocation {
        FailoverAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            revision: self.revision,
            expires_at: self.expires_at,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a failover allocation in status open.
pub fn new_failover_allocation(code: &str, label: &str) -> FailoverAllocation {
    FailoverAllocation {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        revision: 0,
        expires_at: 0,
        region: "".to_string(),
        status: FailoverAllocationStatus::Open,
    }
}
