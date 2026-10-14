use super::*;

use acme_kit::text;

/// Where a global flow log allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalFlowLogAllocationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl GlobalFlowLogAllocationStatus {
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

impl std::fmt::Display for GlobalFlowLogAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global flow log allocation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFlowLogAllocation {
    pub code: String,
    pub title: String,
    pub source: String,
    pub category: String,
    pub discount: f64,
    pub cipher: String,
    pub ceiling: f64,
    pub grace_days: i64,
    pub external_ref: String,
    pub status: GlobalFlowLogAllocationStatus,
}

impl GlobalFlowLogAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalFlowLogAllocationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalFlowLogAllocationStatus) -> bool {
        match self.status {
            GlobalFlowLogAllocationStatus::Open => {
                match next {
                    GlobalFlowLogAllocationStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalFlowLogAllocationStatus::InReview => {
                match next {
                    GlobalFlowLogAllocationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalFlowLogAllocationStatus::Resolved => {
                return next == GlobalFlowLogAllocationStatus::Archived;
            }
            GlobalFlowLogAllocationStatus::Reopened => {
                match next {
                    GlobalFlowLogAllocationStatus::Archived => {
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

    pub fn move_to(&mut self, next: GlobalFlowLogAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global flow log allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.grace_days as f64) * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalFlowLogAllocation {
        GlobalFlowLogAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            category: self.category.clone(),
            discount: self.discount,
            cipher: self.cipher.clone(),
            ceiling: self.ceiling,
            grace_days: self.grace_days,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a global flow log allocation in status open.
pub fn new_global_flow_log_allocation(code: &str, title: &str) -> GlobalFlowLogAllocation {
    GlobalFlowLogAllocation {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        category: "".to_string(),
        discount: 0.0,
        cipher: "".to_string(),
        ceiling: 0.0,
        grace_days: 0,
        external_ref: "".to_string(),
        status: GlobalFlowLogAllocationStatus::Open,
    }
}
