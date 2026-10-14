use super::*;

use acme_kit::text;

/// Where a lenient freeze window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientFreezeWindowStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LenientFreezeWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LenientFreezeWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient freeze window and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeWindow {
    pub code: String,
    pub title: String,
    pub budget: f64,
    pub quantity: i64,
    pub attempts: i64,
    pub archived: bool,
    pub status: LenientFreezeWindowStatus,
}

impl LenientFreezeWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientFreezeWindowStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientFreezeWindowStatus) -> bool {
        match self.status {
            LenientFreezeWindowStatus::Proposed => {
                return next == LenientFreezeWindowStatus::Approved;
            }
            LenientFreezeWindowStatus::Approved => {
                match next {
                    LenientFreezeWindowStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientFreezeWindowStatus::Rejected => {
                match next {
                    LenientFreezeWindowStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: LenientFreezeWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient freeze window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.budget - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientFreezeWindow {
        LenientFreezeWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            budget: self.budget,
            quantity: self.quantity,
            attempts: self.attempts,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient freeze window in status proposed.
pub fn new_lenient_freeze_window(code: &str, title: &str) -> LenientFreezeWindow {
    LenientFreezeWindow {
        code: code.to_string(),
        title: title.to_string(),
        budget: 0.0,
        quantity: 0,
        attempts: 0,
        archived: false,
        status: LenientFreezeWindowStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_lenient_freeze_window("code-642", "title-971");
        assert_eq!(item.status, LenientFreezeWindowStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_lenient_freeze_window("code-642", "title-971");
        if item.move_to(LenientFreezeWindowStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientFreezeWindowStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_lenient_freeze_window("code-642", "title-971");
        item.status = LenientFreezeWindowStatus::Withdrawn;
        assert!(
            item.move_to(LenientFreezeWindowStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
