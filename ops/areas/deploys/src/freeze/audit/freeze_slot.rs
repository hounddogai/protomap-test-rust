use super::*;

use acme_kit::text;

/// Where a freeze slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeSlotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FreezeSlotStatus {
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

impl std::fmt::Display for FreezeSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One freeze slot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeSlot {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub owner_id: String,
    pub score: f64,
    pub threshold: f64,
    pub status: FreezeSlotStatus,
}

impl FreezeSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeSlotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeSlotStatus) -> bool {
        match self.status {
            FreezeSlotStatus::Open => {
                return next == FreezeSlotStatus::Reopened || next == FreezeSlotStatus::Archived;
            }
            FreezeSlotStatus::InReview => {
                match next {
                    FreezeSlotStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeSlotStatus::Resolved => {
                match next {
                    FreezeSlotStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeSlotStatus::Reopened => {
                match next {
                    FreezeSlotStatus::Archived => {
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

    pub fn move_to(&mut self, next: FreezeSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.score + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeSlot {
        FreezeSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            owner_id: self.owner_id.clone(),
            score: self.score,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a freeze slot in status open.
pub fn new_freeze_slot(code: &str, name: &str) -> FreezeSlot {
    FreezeSlot {
        code: code.to_string(),
        name: name.to_string(),
        notes: "".to_string(),
        owner_id: "".to_string(),
        score: 0.0,
        threshold: 0.0,
        status: FreezeSlotStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_freeze_slot("code-236", "name-267");
        assert_eq!(item.status, FreezeSlotStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_freeze_slot("code-236", "name-267");
        if item.move_to(FreezeSlotStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeSlotStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_freeze_slot("code-236", "name-267");
        item.status = FreezeSlotStatus::Archived;
        assert!(item.move_to(FreezeSlotStatus::Open).is_err(), "expected move to fail");
    }
}
