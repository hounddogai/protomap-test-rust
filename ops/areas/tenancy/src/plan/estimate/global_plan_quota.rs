use super::*;

use acme_kit::text;

/// Where a global plan quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalPlanQuotaStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl GlobalPlanQuotaStatus {
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

impl std::fmt::Display for GlobalPlanQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global plan quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanQuota {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub principal: String,
    pub margin: f64,
    pub source: String,
    pub priority: i64,
    pub tags: Vec<String>,
    pub status: GlobalPlanQuotaStatus,
}

impl GlobalPlanQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalPlanQuotaStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalPlanQuotaStatus) -> bool {
        match self.status {
            GlobalPlanQuotaStatus::Open => {
                return next == GlobalPlanQuotaStatus::Resolved;
            }
            GlobalPlanQuotaStatus::InReview => {
                match next {
                    GlobalPlanQuotaStatus::Reopened | GlobalPlanQuotaStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalPlanQuotaStatus::Resolved => {
                return next == GlobalPlanQuotaStatus::Reopened || next == GlobalPlanQuotaStatus::Archived;
            }
            GlobalPlanQuotaStatus::Reopened => {
                return next == GlobalPlanQuotaStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalPlanQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global plan quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.priority as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalPlanQuota {
        GlobalPlanQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            principal: self.principal.clone(),
            margin: self.margin,
            source: self.source.clone(),
            priority: self.priority,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a global plan quota in status open.
pub fn new_global_plan_quota(code: &str, name: &str) -> GlobalPlanQuota {
    GlobalPlanQuota {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        principal: "".to_string(),
        margin: 0.0,
        source: "".to_string(),
        priority: 0,
        tags: Vec::new(),
        status: GlobalPlanQuotaStatus::Open,
    }
}
