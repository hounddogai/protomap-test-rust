use super::*;

use acme_kit::text;

/// Where a lenient quote milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientQuoteMilestoneStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientQuoteMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientQuoteMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient quote milestone and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientQuoteMilestone {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub priority: i64,
    pub amount: f64,
    pub issuer: String,
    pub ceiling: f64,
    pub status: LenientQuoteMilestoneStatus,
}

impl LenientQuoteMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientQuoteMilestoneStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientQuoteMilestoneStatus) -> bool {
        match self.status {
            LenientQuoteMilestoneStatus::Created => {
                match next {
                    LenientQuoteMilestoneStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientQuoteMilestoneStatus::Validated => {
                match next {
                    LenientQuoteMilestoneStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientQuoteMilestoneStatus::Published => {
                return next == LenientQuoteMilestoneStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientQuoteMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient quote milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient quote milestone", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.amount - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientQuoteMilestone {
        LenientQuoteMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            priority: self.priority,
            amount: self.amount,
            issuer: self.issuer.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a lenient quote milestone in status created.
pub fn new_lenient_quote_milestone(code: &str, name: &str) -> LenientQuoteMilestone {
    LenientQuoteMilestone {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        priority: 0,
        amount: 0.0,
        issuer: "".to_string(),
        ceiling: 0.0,
        status: LenientQuoteMilestoneStatus::Created,
    }
}
