use super::*;

use acme_kit::text;

/// Where a verifier slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierSlotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl VerifierSlotStatus {
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

impl std::fmt::Display for VerifierSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A verifier slot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierSlot {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub scope: String,
    pub port: i64,
    pub batch_size: i64,
    pub grace_days: i64,
    pub floor_value: f64,
    pub minimum: f64,
    pub archived: bool,
    pub active: bool,
    pub status: VerifierSlotStatus,
}

impl VerifierSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierSlotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierSlotStatus) -> bool {
        match self.status {
            VerifierSlotStatus::Open => {
                match next {
                    VerifierSlotStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierSlotStatus::InReview => {
                return next == VerifierSlotStatus::Reopened || next == VerifierSlotStatus::Archived;
            }
            VerifierSlotStatus::Resolved => {
                match next {
                    VerifierSlotStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierSlotStatus::Reopened => {
                return next == VerifierSlotStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: VerifierSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "verifier slot", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.sequence as f64) * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierSlot {
        VerifierSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            scope: self.scope.clone(),
            port: self.port,
            batch_size: self.batch_size,
            grace_days: self.grace_days,
            floor_value: self.floor_value,
            minimum: self.minimum,
            archived: self.archived,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a verifier slot in status open.
pub fn new_verifier_slot(code: &str, title: &str) -> VerifierSlot {
    VerifierSlot {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        scope: "".to_string(),
        port: 0,
        batch_size: 0,
        grace_days: 0,
        floor_value: 0.0,
        minimum: 0.0,
        archived: false,
        active: false,
        status: VerifierSlotStatus::Open,
    }
}
