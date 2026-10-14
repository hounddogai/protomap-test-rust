use super::*;

use acme_kit::text;

/// Where a manual peering reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualPeeringReservationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ManualPeeringReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for ManualPeeringReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual peering reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualPeeringReservation {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub cipher: String,
    pub attempts: i64,
    pub grace_days: i64,
    pub port: i64,
    pub checksum: String,
    pub status: ManualPeeringReservationStatus,
}

impl ManualPeeringReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualPeeringReservationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualPeeringReservationStatus) -> bool {
        match self.status {
            ManualPeeringReservationStatus::Planned => {
                match next {
                    ManualPeeringReservationStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualPeeringReservationStatus::Scheduled => {
                return next == ManualPeeringReservationStatus::Completed
                    || next == ManualPeeringReservationStatus::Cancelled;
            }
            ManualPeeringReservationStatus::InProgress => {
                match next {
                    ManualPeeringReservationStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualPeeringReservationStatus::Completed => {
                return next == ManualPeeringReservationStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualPeeringReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual peering reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.grace_days as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualPeeringReservation {
        ManualPeeringReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            cipher: self.cipher.clone(),
            attempts: self.attempts,
            grace_days: self.grace_days,
            port: self.port,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a manual peering reservation in status planned.
pub fn new_manual_peering_reservation(code: &str, label: &str) -> ManualPeeringReservation {
    ManualPeeringReservation {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        cipher: "".to_string(),
        attempts: 0,
        grace_days: 0,
        port: 0,
        checksum: "".to_string(),
        status: ManualPeeringReservationStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_manual_peering_reservation("code-364", "label-957");
        assert_eq!(item.status, ManualPeeringReservationStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_manual_peering_reservation("code-364", "label-957");
        if item.move_to(ManualPeeringReservationStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualPeeringReservationStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_manual_peering_reservation("code-364", "label-957");
        item.status = ManualPeeringReservationStatus::Cancelled;
        assert!(
            item.move_to(ManualPeeringReservationStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
