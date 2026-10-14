use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a timeline reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineReservationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TimelineReservationStatus {
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

impl std::fmt::Display for TimelineReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A timeline reservation as the timeline team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineReservation {
    pub code: String,
    pub name: String,
    pub budget: f64,
    pub cipher: String,
    pub fingerprint: String,
    pub attributes: BTreeMap<String, String>,
    pub status: TimelineReservationStatus,
}

impl TimelineReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineReservationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineReservationStatus) -> bool {
        match self.status {
            TimelineReservationStatus::Open => {
                return next == TimelineReservationStatus::Archived;
            }
            TimelineReservationStatus::InReview => {
                return next == TimelineReservationStatus::Reopened;
            }
            TimelineReservationStatus::Resolved => {
                return next == TimelineReservationStatus::Reopened;
            }
            TimelineReservationStatus::Reopened => {
                return next == TimelineReservationStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TimelineReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.budget
    }

    pub fn copy(&self) -> TimelineReservation {
        TimelineReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            budget: self.budget,
            cipher: self.cipher.clone(),
            fingerprint: self.fingerprint.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a timeline reservation in status open.
pub fn new_timeline_reservation(code: &str, name: &str) -> TimelineReservation {
    TimelineReservation {
        code: code.to_string(),
        name: name.to_string(),
        budget: 0.0,
        cipher: "".to_string(),
        fingerprint: "".to_string(),
        attributes: BTreeMap::new(),
        status: TimelineReservationStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_timeline_reservation("code-97", "name-342");
        assert_eq!(item.status, TimelineReservationStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_timeline_reservation("code-97", "name-342");
        if item.move_to(TimelineReservationStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineReservationStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_timeline_reservation("code-97", "name-342");
        item.status = TimelineReservationStatus::Archived;
        assert!(
            item.move_to(TimelineReservationStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
