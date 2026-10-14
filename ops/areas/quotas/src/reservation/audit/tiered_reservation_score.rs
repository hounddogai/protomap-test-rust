use super::*;

use acme_kit::text;

/// Where a tiered reservation score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredReservationScoreStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TieredReservationScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TieredReservationScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered reservation score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredReservationScore {
    pub code: String,
    pub label: String,
    pub source: String,
    pub port: i64,
    pub audience: String,
    pub status: TieredReservationScoreStatus,
}

impl TieredReservationScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredReservationScoreStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredReservationScoreStatus) -> bool {
        match self.status {
            TieredReservationScoreStatus::Proposed => {
                return next == TieredReservationScoreStatus::Rejected
                    || next == TieredReservationScoreStatus::Withdrawn;
            }
            TieredReservationScoreStatus::Approved => {
                match next {
                    TieredReservationScoreStatus::Rejected | TieredReservationScoreStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredReservationScoreStatus::Rejected => {
                match next {
                    TieredReservationScoreStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: TieredReservationScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered reservation score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tiered reservation score", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.port as f64
    }

    pub fn copy(&self) -> TieredReservationScore {
        TieredReservationScore {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            port: self.port,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a tiered reservation score in status proposed.
pub fn new_tiered_reservation_score(code: &str, label: &str) -> TieredReservationScore {
    TieredReservationScore {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        port: 0,
        audience: "".to_string(),
        status: TieredReservationScoreStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_tiered_reservation_score("code-612", "label-217");
        assert_eq!(item.status, TieredReservationScoreStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_tiered_reservation_score("code-612", "label-217");
        if item.move_to(TieredReservationScoreStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredReservationScoreStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_tiered_reservation_score("code-612", "label-217");
        item.status = TieredReservationScoreStatus::Withdrawn;
        assert!(
            item.move_to(TieredReservationScoreStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
