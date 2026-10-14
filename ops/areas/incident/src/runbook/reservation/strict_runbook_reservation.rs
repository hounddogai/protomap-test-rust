use super::*;

use acme_kit::text;

/// Where a strict runbook reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictRunbookReservationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl StrictRunbookReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for StrictRunbookReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict runbook reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRunbookReservation {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub notes: String,
    pub revision: i64,
    pub scope: String,
    pub archived: bool,
    pub pinned: bool,
    pub margin: f64,
    pub status: StrictRunbookReservationStatus,
}

impl StrictRunbookReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictRunbookReservationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictRunbookReservationStatus) -> bool {
        match self.status {
            StrictRunbookReservationStatus::Draft => {
                match next {
                    StrictRunbookReservationStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictRunbookReservationStatus::Pending => {
                match next {
                    StrictRunbookReservationStatus::Active | StrictRunbookReservationStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictRunbookReservationStatus::Active => {
                match next {
                    StrictRunbookReservationStatus::Suspended | StrictRunbookReservationStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictRunbookReservationStatus::Suspended => {
                return next == StrictRunbookReservationStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictRunbookReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict runbook reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.margin - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictRunbookReservation {
        StrictRunbookReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            notes: self.notes.clone(),
            revision: self.revision,
            scope: self.scope.clone(),
            archived: self.archived,
            pinned: self.pinned,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a strict runbook reservation in status draft.
pub fn new_strict_runbook_reservation(code: &str, name: &str) -> StrictRunbookReservation {
    StrictRunbookReservation {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        notes: "".to_string(),
        revision: 0,
        scope: "".to_string(),
        archived: false,
        pinned: false,
        margin: 0.0,
        status: StrictRunbookReservationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_strict_runbook_reservation("code-991", "name-394");
        assert_eq!(item.status, StrictRunbookReservationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_strict_runbook_reservation("code-991", "name-394");
        if item.move_to(StrictRunbookReservationStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictRunbookReservationStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_strict_runbook_reservation("code-991", "name-394");
        item.status = StrictRunbookReservationStatus::Closed;
        assert!(
            item.move_to(StrictRunbookReservationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
