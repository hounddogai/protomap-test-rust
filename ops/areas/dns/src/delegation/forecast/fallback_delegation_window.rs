use super::*;

use acme_kit::text;

/// Where a fallback delegation window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackDelegationWindowStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackDelegationWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for FallbackDelegationWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback delegation window. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDelegationWindow {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub score: f64,
    pub tenant_id: String,
    pub description: String,
    pub budget: f64,
    pub contact_email: String,
    pub retries: i64,
    pub summary: String,
    pub status: FallbackDelegationWindowStatus,
}

impl FallbackDelegationWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackDelegationWindowStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackDelegationWindowStatus) -> bool {
        match self.status {
            FallbackDelegationWindowStatus::New => {
                match next {
                    FallbackDelegationWindowStatus::Queued | FallbackDelegationWindowStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackDelegationWindowStatus::Queued => {
                return next == FallbackDelegationWindowStatus::Failed;
            }
            FallbackDelegationWindowStatus::Running => {
                return next == FallbackDelegationWindowStatus::Done;
            }
            FallbackDelegationWindowStatus::Done => {
                match next {
                    FallbackDelegationWindowStatus::Failed => {
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

    pub fn move_to(&mut self, next: FallbackDelegationWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback delegation window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "fallback delegation window", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.retries as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackDelegationWindow {
        FallbackDelegationWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            audience: self.audience.clone(),
            score: self.score,
            tenant_id: self.tenant_id.clone(),
            description: self.description.clone(),
            budget: self.budget,
            contact_email: self.contact_email.clone(),
            retries: self.retries,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a fallback delegation window in status new.
pub fn new_fallback_delegation_window(code: &str, label: &str) -> FallbackDelegationWindow {
    FallbackDelegationWindow {
        code: code.to_string(),
        label: label.to_string(),
        audience: "".to_string(),
        score: 0.0,
        tenant_id: "".to_string(),
        description: "".to_string(),
        budget: 0.0,
        contact_email: "".to_string(),
        retries: 0,
        summary: "".to_string(),
        status: FallbackDelegationWindowStatus::New,
    }
}
