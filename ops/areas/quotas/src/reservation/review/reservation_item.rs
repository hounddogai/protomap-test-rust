use super::*;

use acme_kit::text;

/// Where a reservation item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationItemStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ReservationItemStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ReservationItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a reservation item and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationItem {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub updated_at: i64,
    pub revision: i64,
    pub status: ReservationItemStatus,
}

impl ReservationItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationItemStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationItemStatus) -> bool {
        match self.status {
            ReservationItemStatus::Created => {
                return next == ReservationItemStatus::Published || next == ReservationItemStatus::Retired;
            }
            ReservationItemStatus::Validated => {
                return next == ReservationItemStatus::Retired;
            }
            ReservationItemStatus::Published => {
                return next == ReservationItemStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation item {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.updated_at as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationItem {
        ReservationItem {
            code: self.code.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            updated_at: self.updated_at,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a reservation item in status created.
pub fn new_reservation_item(code: &str, title: &str) -> ReservationItem {
    ReservationItem {
        code: code.to_string(),
        title: title.to_string(),
        notes: "".to_string(),
        updated_at: 0,
        revision: 0,
        status: ReservationItemStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_reservation_item("code-120", "title-591");
        assert_eq!(item.status, ReservationItemStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_reservation_item("code-120", "title-591");
        if item.move_to(ReservationItemStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationItemStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_reservation_item("code-120", "title-591");
        item.status = ReservationItemStatus::Retired;
        assert!(
            item.move_to(ReservationItemStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
