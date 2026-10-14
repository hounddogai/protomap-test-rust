use super::*;

use acme_kit::text;

/// Where a strict silence reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictSilenceReservationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl StrictSilenceReservationStatus {
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

impl std::fmt::Display for StrictSilenceReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict silence reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictSilenceReservation {
    pub code: String,
    pub title: String,
    pub region: String,
    pub checksum: String,
    pub source: String,
    pub capacity: i64,
    pub port: i64,
    pub retries: i64,
    pub version: i64,
    pub summary: String,
    pub sequence: i64,
    pub status: StrictSilenceReservationStatus,
}

impl StrictSilenceReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictSilenceReservationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictSilenceReservationStatus) -> bool {
        match self.status {
            StrictSilenceReservationStatus::Open => {
                match next {
                    StrictSilenceReservationStatus::Resolved | StrictSilenceReservationStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictSilenceReservationStatus::InReview => {
                return next == StrictSilenceReservationStatus::Resolved
                    || next == StrictSilenceReservationStatus::Reopened;
            }
            StrictSilenceReservationStatus::Resolved => {
                return next == StrictSilenceReservationStatus::Reopened;
            }
            StrictSilenceReservationStatus::Reopened => {
                return next == StrictSilenceReservationStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictSilenceReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict silence reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict silence reservation", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.retries as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictSilenceReservation {
        StrictSilenceReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            checksum: self.checksum.clone(),
            source: self.source.clone(),
            capacity: self.capacity,
            port: self.port,
            retries: self.retries,
            version: self.version,
            summary: self.summary.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a strict silence reservation in status open.
pub fn new_strict_silence_reservation(code: &str, title: &str) -> StrictSilenceReservation {
    StrictSilenceReservation {
        code: code.to_string(),
        title: title.to_string(),
        region: "".to_string(),
        checksum: "".to_string(),
        source: "".to_string(),
        capacity: 0,
        port: 0,
        retries: 0,
        version: 0,
        summary: "".to_string(),
        sequence: 0,
        status: StrictSilenceReservationStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_strict_silence_reservation("code-95", "title-521");
        assert_eq!(item.status, StrictSilenceReservationStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_strict_silence_reservation("code-95", "title-521");
        if item.move_to(StrictSilenceReservationStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictSilenceReservationStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_strict_silence_reservation("code-95", "title-521");
        item.status = StrictSilenceReservationStatus::Archived;
        assert!(
            item.move_to(StrictSilenceReservationStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
