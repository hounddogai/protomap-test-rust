use super::*;

use acme_kit::text;

/// Where a primary dashboard reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryDashboardReservationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryDashboardReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryDashboardReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary dashboard reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDashboardReservation {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub unit: String,
    pub starts_at: i64,
    pub audience: String,
    pub batch_size: i64,
    pub status: PrimaryDashboardReservationStatus,
}

impl PrimaryDashboardReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryDashboardReservationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryDashboardReservationStatus) -> bool {
        match self.status {
            PrimaryDashboardReservationStatus::Requested => {
                match next {
                    PrimaryDashboardReservationStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryDashboardReservationStatus::Granted => {
                return next == PrimaryDashboardReservationStatus::Revoked
                    || next == PrimaryDashboardReservationStatus::Expired;
            }
            PrimaryDashboardReservationStatus::Revoked => {
                return next == PrimaryDashboardReservationStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryDashboardReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary dashboard reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.batch_size as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryDashboardReservation {
        PrimaryDashboardReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            unit: self.unit.clone(),
            starts_at: self.starts_at,
            audience: self.audience.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a primary dashboard reservation in status requested.
pub fn new_primary_dashboard_reservation(code: &str, title: &str) -> PrimaryDashboardReservation {
    PrimaryDashboardReservation {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        unit: "".to_string(),
        starts_at: 0,
        audience: "".to_string(),
        batch_size: 0,
        status: PrimaryDashboardReservationStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_primary_dashboard_reservation("code-521", "title-613");
        assert_eq!(item.status, PrimaryDashboardReservationStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_primary_dashboard_reservation("code-521", "title-613");
        if item.move_to(PrimaryDashboardReservationStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryDashboardReservationStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_primary_dashboard_reservation("code-521", "title-613");
        item.status = PrimaryDashboardReservationStatus::Expired;
        assert!(
            item.move_to(PrimaryDashboardReservationStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
