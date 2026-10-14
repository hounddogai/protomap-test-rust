use super::*;

use acme_kit::text;

/// Where a lenient approval budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientApprovalBudgetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientApprovalBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for LenientApprovalBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient approval budget as the approval team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientApprovalBudget {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub unit: String,
    pub rotation_days: i64,
    pub tags: Vec<String>,
    pub source: String,
    pub principal: String,
    pub status: LenientApprovalBudgetStatus,
}

impl LenientApprovalBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientApprovalBudgetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientApprovalBudgetStatus) -> bool {
        match self.status {
            LenientApprovalBudgetStatus::Idle => {
                match next {
                    LenientApprovalBudgetStatus::Draining | LenientApprovalBudgetStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientApprovalBudgetStatus::Warming => {
                return next == LenientApprovalBudgetStatus::Ready || next == LenientApprovalBudgetStatus::Draining;
            }
            LenientApprovalBudgetStatus::Ready => {
                match next {
                    LenientApprovalBudgetStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientApprovalBudgetStatus::Draining => {
                return next == LenientApprovalBudgetStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientApprovalBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient approval budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient approval budget", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.rotation_days as f64
    }

    pub fn copy(&self) -> LenientApprovalBudget {
        LenientApprovalBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            unit: self.unit.clone(),
            rotation_days: self.rotation_days,
            tags: self.tags.clone(),
            source: self.source.clone(),
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a lenient approval budget in status idle.
pub fn new_lenient_approval_budget(code: &str, title: &str) -> LenientApprovalBudget {
    LenientApprovalBudget {
        code: code.to_string(),
        title: title.to_string(),
        timezone: "".to_string(),
        unit: "".to_string(),
        rotation_days: 0,
        tags: Vec::new(),
        source: "".to_string(),
        principal: "".to_string(),
        status: LenientApprovalBudgetStatus::Idle,
    }
}
