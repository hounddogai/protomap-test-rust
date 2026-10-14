use super::*;

use acme_kit::text;

/// Where a lenient reservation digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientReservationDigestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LenientReservationDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LenientReservationDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient reservation digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationDigest {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub revision: i64,
    pub approved: bool,
    pub weight: f64,
    pub region: String,
    pub version: i64,
    pub reviewer: String,
    pub host: String,
    pub audience: String,
    pub status: LenientReservationDigestStatus,
}

impl LenientReservationDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientReservationDigestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientReservationDigestStatus) -> bool {
        match self.status {
            LenientReservationDigestStatus::Proposed => {
                match next {
                    LenientReservationDigestStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientReservationDigestStatus::Approved => {
                return next == LenientReservationDigestStatus::Rejected;
            }
            LenientReservationDigestStatus::Rejected => {
                match next {
                    LenientReservationDigestStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: LenientReservationDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient reservation digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.revision as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientReservationDigest {
        LenientReservationDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            revision: self.revision,
            approved: self.approved,
            weight: self.weight,
            region: self.region.clone(),
            version: self.version,
            reviewer: self.reviewer.clone(),
            host: self.host.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a lenient reservation digest in status proposed.
pub fn new_lenient_reservation_digest(code: &str, title: &str) -> LenientReservationDigest {
    LenientReservationDigest {
        code: code.to_string(),
        title: title.to_string(),
        summary: "".to_string(),
        revision: 0,
        approved: false,
        weight: 0.0,
        region: "".to_string(),
        version: 0,
        reviewer: "".to_string(),
        host: "".to_string(),
        audience: "".to_string(),
        status: LenientReservationDigestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_lenient_reservation_digest("code-526", "title-728");
        assert_eq!(item.status, LenientReservationDigestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_lenient_reservation_digest("code-526", "title-728");
        if item.move_to(LenientReservationDigestStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientReservationDigestStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_lenient_reservation_digest("code-526", "title-728");
        item.status = LenientReservationDigestStatus::Withdrawn;
        assert!(
            item.move_to(LenientReservationDigestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
