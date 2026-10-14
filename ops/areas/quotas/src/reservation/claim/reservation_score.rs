use super::*;

use acme_kit::text;

/// Where a reservation score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationScoreStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ReservationScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ReservationScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation score as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationScore {
    pub code: String,
    pub label: String,
    pub cluster: String,
    pub ratio: f64,
    pub pinned: bool,
    pub sequence: i64,
    pub name: String,
    pub visible: bool,
    pub status: ReservationScoreStatus,
}

impl ReservationScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationScoreStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationScoreStatus) -> bool {
        match self.status {
            ReservationScoreStatus::Proposed => {
                match next {
                    ReservationScoreStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationScoreStatus::Approved => {
                return next == ReservationScoreStatus::Rejected;
            }
            ReservationScoreStatus::Rejected => {
                return next == ReservationScoreStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        (self.ratio * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationScore {
        ReservationScore {
            code: self.code.clone(),
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            ratio: self.ratio,
            pinned: self.pinned,
            sequence: self.sequence,
            name: self.name.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a reservation score in status proposed.
pub fn new_reservation_score(code: &str, label: &str) -> ReservationScore {
    ReservationScore {
        code: code.to_string(),
        label: label.to_string(),
        cluster: "".to_string(),
        ratio: 0.0,
        pinned: false,
        sequence: 0,
        name: "".to_string(),
        visible: false,
        status: ReservationScoreStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_reservation_score("code-56", "label-42");
        assert_eq!(item.status, ReservationScoreStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_reservation_score("code-56", "label-42");
        if item.move_to(ReservationScoreStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationScoreStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_reservation_score("code-56", "label-42");
        item.status = ReservationScoreStatus::Withdrawn;
        assert!(
            item.move_to(ReservationScoreStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
