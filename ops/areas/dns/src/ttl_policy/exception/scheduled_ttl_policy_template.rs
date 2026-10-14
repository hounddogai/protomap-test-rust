use super::*;

use acme_kit::text;

/// Where a scheduled ttl policy template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledTtlPolicyTemplateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ScheduledTtlPolicyTemplateStatus {
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

impl std::fmt::Display for ScheduledTtlPolicyTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled ttl policy template as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTtlPolicyTemplate {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub country: String,
    pub reviewer: String,
    pub visible: bool,
    pub tenant_id: String,
    pub segment: String,
    pub status: ScheduledTtlPolicyTemplateStatus,
}

impl ScheduledTtlPolicyTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledTtlPolicyTemplateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledTtlPolicyTemplateStatus) -> bool {
        match self.status {
            ScheduledTtlPolicyTemplateStatus::Open => {
                return next == ScheduledTtlPolicyTemplateStatus::Reopened
                    || next == ScheduledTtlPolicyTemplateStatus::Archived;
            }
            ScheduledTtlPolicyTemplateStatus::InReview => {
                return next == ScheduledTtlPolicyTemplateStatus::Resolved
                    || next == ScheduledTtlPolicyTemplateStatus::Reopened;
            }
            ScheduledTtlPolicyTemplateStatus::Resolved => {
                match next {
                    ScheduledTtlPolicyTemplateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledTtlPolicyTemplateStatus::Reopened => {
                return next == ScheduledTtlPolicyTemplateStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledTtlPolicyTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled ttl policy template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> ScheduledTtlPolicyTemplate {
        ScheduledTtlPolicyTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            country: self.country.clone(),
            reviewer: self.reviewer.clone(),
            visible: self.visible,
            tenant_id: self.tenant_id.clone(),
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a scheduled ttl policy template in status open.
pub fn new_scheduled_ttl_policy_template(code: &str, name: &str) -> ScheduledTtlPolicyTemplate {
    ScheduledTtlPolicyTemplate {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        country: "".to_string(),
        reviewer: "".to_string(),
        visible: false,
        tenant_id: "".to_string(),
        segment: "".to_string(),
        status: ScheduledTtlPolicyTemplateStatus::Open,
    }
}
