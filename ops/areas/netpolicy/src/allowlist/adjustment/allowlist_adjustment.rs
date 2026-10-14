use super::*;

use acme_kit::text;

/// Where a allowlist adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistAdjustmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AllowlistAdjustmentStatus {
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

impl std::fmt::Display for AllowlistAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a allowlist adjustment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistAdjustment {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub score: f64,
    pub source: String,
    pub ceiling: f64,
    pub channel: String,
    pub archived: bool,
    pub quantity: i64,
    pub status: AllowlistAdjustmentStatus,
}

impl AllowlistAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistAdjustmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistAdjustmentStatus) -> bool {
        match self.status {
            AllowlistAdjustmentStatus::Open => {
                return next == AllowlistAdjustmentStatus::Resolved || next == AllowlistAdjustmentStatus::Reopened;
            }
            AllowlistAdjustmentStatus::InReview => {
                return next == AllowlistAdjustmentStatus::Archived;
            }
            AllowlistAdjustmentStatus::Resolved => {
                match next {
                    AllowlistAdjustmentStatus::Reopened | AllowlistAdjustmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistAdjustmentStatus::Reopened => {
                match next {
                    AllowlistAdjustmentStatus::Archived => {
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

    pub fn move_to(&mut self, next: AllowlistAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.score - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistAdjustment {
        AllowlistAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            score: self.score,
            source: self.source.clone(),
            ceiling: self.ceiling,
            channel: self.channel.clone(),
            archived: self.archived,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a allowlist adjustment in status open.
pub fn new_allowlist_adjustment(code: &str, name: &str) -> AllowlistAdjustment {
    AllowlistAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        score: 0.0,
        source: "".to_string(),
        ceiling: 0.0,
        channel: "".to_string(),
        archived: false,
        quantity: 0,
        status: AllowlistAdjustmentStatus::Open,
    }
}
