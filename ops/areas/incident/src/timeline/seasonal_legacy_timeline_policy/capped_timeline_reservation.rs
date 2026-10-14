use super::*;

use acme_kit::text;

/// Where a capped timeline reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedTimelineReservationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CappedTimelineReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CappedTimelineReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped timeline reservation as the timeline team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedTimelineReservation {
    pub code: String,
    pub title: String,
    pub contact_email: String,
    pub margin: f64,
    pub burst: i64,
    pub status: CappedTimelineReservationStatus,
}

impl CappedTimelineReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedTimelineReservationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedTimelineReservationStatus) -> bool {
        match self.status {
            CappedTimelineReservationStatus::Proposed => {
                match next {
                    CappedTimelineReservationStatus::Approved | CappedTimelineReservationStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedTimelineReservationStatus::Approved => {
                return next == CappedTimelineReservationStatus::Rejected
                    || next == CappedTimelineReservationStatus::Withdrawn;
            }
            CappedTimelineReservationStatus::Rejected => {
                return next == CappedTimelineReservationStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedTimelineReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped timeline reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.burst as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedTimelineReservation {
        CappedTimelineReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            margin: self.margin,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a capped timeline reservation in status proposed.
pub fn new_capped_timeline_reservation(code: &str, title: &str) -> CappedTimelineReservation {
    CappedTimelineReservation {
        code: code.to_string(),
        title: title.to_string(),
        contact_email: "".to_string(),
        margin: 0.0,
        burst: 0,
        status: CappedTimelineReservationStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_capped_timeline_reservation("code-632", "title-840");
        assert_eq!(item.status, CappedTimelineReservationStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_capped_timeline_reservation("code-632", "title-840");
        if item.move_to(CappedTimelineReservationStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedTimelineReservationStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_capped_timeline_reservation("code-632", "title-840");
        item.status = CappedTimelineReservationStatus::Withdrawn;
        assert!(
            item.move_to(CappedTimelineReservationStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
