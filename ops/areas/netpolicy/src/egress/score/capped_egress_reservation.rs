use super::*;

use acme_kit::text;

/// Where a capped egress reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedEgressReservationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl CappedEgressReservationStatus {
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

impl std::fmt::Display for CappedEgressReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped egress reservation as the egress team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedEgressReservation {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub checksum: String,
    pub version: i64,
    pub owner_id: String,
    pub status: CappedEgressReservationStatus,
}

impl CappedEgressReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedEgressReservationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedEgressReservationStatus) -> bool {
        match self.status {
            CappedEgressReservationStatus::Planned => {
                match next {
                    CappedEgressReservationStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedEgressReservationStatus::Scheduled => {
                return next == CappedEgressReservationStatus::Cancelled;
            }
            CappedEgressReservationStatus::InProgress => {
                return next == CappedEgressReservationStatus::Completed
                    || next == CappedEgressReservationStatus::Cancelled;
            }
            CappedEgressReservationStatus::Completed => {
                match next {
                    CappedEgressReservationStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: CappedEgressReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped egress reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.version as f64 - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedEgressReservation {
        CappedEgressReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            checksum: self.checksum.clone(),
            version: self.version,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a capped egress reservation in status planned.
pub fn new_capped_egress_reservation(code: &str, name: &str) -> CappedEgressReservation {
    CappedEgressReservation {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        checksum: "".to_string(),
        version: 0,
        owner_id: "".to_string(),
        status: CappedEgressReservationStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_capped_egress_reservation("code-962", "name-800");
        assert_eq!(item.status, CappedEgressReservationStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_capped_egress_reservation("code-962", "name-800");
        if item.move_to(CappedEgressReservationStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedEgressReservationStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_capped_egress_reservation("code-962", "name-800");
        item.status = CappedEgressReservationStatus::Cancelled;
        assert!(
            item.move_to(CappedEgressReservationStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
