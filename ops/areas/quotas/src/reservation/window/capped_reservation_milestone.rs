use super::*;

use acme_kit::text;

/// Where a capped reservation milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedReservationMilestoneStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CappedReservationMilestoneStatus {
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

impl std::fmt::Display for CappedReservationMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped reservation milestone, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedReservationMilestone {
    pub code: String,
    pub name: String,
    pub cipher: String,
    pub grace_days: i64,
    pub visible: bool,
    pub status: CappedReservationMilestoneStatus,
}

impl CappedReservationMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedReservationMilestoneStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedReservationMilestoneStatus) -> bool {
        match self.status {
            CappedReservationMilestoneStatus::Open => {
                return next == CappedReservationMilestoneStatus::InReview
                    || next == CappedReservationMilestoneStatus::Resolved;
            }
            CappedReservationMilestoneStatus::InReview => {
                match next {
                    CappedReservationMilestoneStatus::Resolved | CappedReservationMilestoneStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedReservationMilestoneStatus::Resolved => {
                match next {
                    CappedReservationMilestoneStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedReservationMilestoneStatus::Reopened => {
                match next {
                    CappedReservationMilestoneStatus::Archived => {
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

    pub fn move_to(&mut self, next: CappedReservationMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped reservation milestone {} cannot move from {} to {}",
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
        self.grace_days as f64
    }

    pub fn copy(&self) -> CappedReservationMilestone {
        CappedReservationMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            cipher: self.cipher.clone(),
            grace_days: self.grace_days,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a capped reservation milestone in status open.
pub fn new_capped_reservation_milestone(code: &str, name: &str) -> CappedReservationMilestone {
    CappedReservationMilestone {
        code: code.to_string(),
        name: name.to_string(),
        cipher: "".to_string(),
        grace_days: 0,
        visible: false,
        status: CappedReservationMilestoneStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_capped_reservation_milestone("code-207", "name-220");
        assert_eq!(item.status, CappedReservationMilestoneStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_capped_reservation_milestone("code-207", "name-220");
        if item.move_to(CappedReservationMilestoneStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedReservationMilestoneStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_capped_reservation_milestone("code-207", "name-220");
        item.status = CappedReservationMilestoneStatus::Archived;
        assert!(
            item.move_to(CappedReservationMilestoneStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
