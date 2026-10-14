use super::*;

use acme_kit::text;

/// Where a release reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseReservationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ReleaseReservationStatus {
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

impl std::fmt::Display for ReleaseReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A release reservation as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseReservation {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub external_ref: String,
    pub principal: String,
    pub label: String,
    pub status: ReleaseReservationStatus,
}

impl ReleaseReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseReservationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseReservationStatus) -> bool {
        match self.status {
            ReleaseReservationStatus::Draft => {
                match next {
                    ReleaseReservationStatus::Active | ReleaseReservationStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseReservationStatus::Pending => {
                match next {
                    ReleaseReservationStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseReservationStatus::Active => {
                return next == ReleaseReservationStatus::Closed;
            }
            ReleaseReservationStatus::Suspended => {
                return next == ReleaseReservationStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleaseReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.sequence as f64
    }

    pub fn copy(&self) -> ReleaseReservation {
        ReleaseReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            external_ref: self.external_ref.clone(),
            principal: self.principal.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a release reservation in status draft.
pub fn new_release_reservation(code: &str, title: &str) -> ReleaseReservation {
    ReleaseReservation {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        external_ref: "".to_string(),
        principal: "".to_string(),
        label: "".to_string(),
        status: ReleaseReservationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_release_reservation("code-57", "title-515");
        assert_eq!(item.status, ReleaseReservationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_release_reservation("code-57", "title-515");
        if item.move_to(ReleaseReservationStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseReservationStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_release_reservation("code-57", "title-515");
        item.status = ReleaseReservationStatus::Closed;
        assert!(
            item.move_to(ReleaseReservationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
