use super::*;

use acme_kit::text;

/// Where a reservation manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationManifestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ReservationManifestStatus {
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

impl std::fmt::Display for ReservationManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationManifest {
    pub code: String,
    pub title: String,
    pub source: String,
    pub currency: String,
    pub checksum: String,
    pub burst: i64,
    pub ratio: f64,
    pub visible: bool,
    pub timezone: String,
    pub external_ref: String,
    pub status: ReservationManifestStatus,
}

impl ReservationManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationManifestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationManifestStatus) -> bool {
        match self.status {
            ReservationManifestStatus::Open => {
                return next == ReservationManifestStatus::Reopened || next == ReservationManifestStatus::Archived;
            }
            ReservationManifestStatus::InReview => {
                match next {
                    ReservationManifestStatus::Resolved | ReservationManifestStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationManifestStatus::Resolved => {
                return next == ReservationManifestStatus::Reopened || next == ReservationManifestStatus::Archived;
            }
            ReservationManifestStatus::Reopened => {
                return next == ReservationManifestStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.burst as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationManifest {
        ReservationManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            currency: self.currency.clone(),
            checksum: self.checksum.clone(),
            burst: self.burst,
            ratio: self.ratio,
            visible: self.visible,
            timezone: self.timezone.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a reservation manifest in status open.
pub fn new_reservation_manifest(code: &str, title: &str) -> ReservationManifest {
    ReservationManifest {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        currency: "".to_string(),
        checksum: "".to_string(),
        burst: 0,
        ratio: 0.0,
        visible: false,
        timezone: "".to_string(),
        external_ref: "".to_string(),
        status: ReservationManifestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_reservation_manifest("code-380", "title-319");
        assert_eq!(item.status, ReservationManifestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_reservation_manifest("code-380", "title-319");
        if item.move_to(ReservationManifestStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationManifestStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_reservation_manifest("code-380", "title-319");
        item.status = ReservationManifestStatus::Archived;
        assert!(
            item.move_to(ReservationManifestStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
