use super::*;

use acme_kit::text;

/// Where a dashboard reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardReservationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DashboardReservationStatus {
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

impl std::fmt::Display for DashboardReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a dashboard reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardReservation {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub title: String,
    pub checksum: String,
    pub namespace: String,
    pub quantity: i64,
    pub status: DashboardReservationStatus,
}

impl DashboardReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardReservationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardReservationStatus) -> bool {
        match self.status {
            DashboardReservationStatus::Open => {
                return next == DashboardReservationStatus::Reopened || next == DashboardReservationStatus::Archived;
            }
            DashboardReservationStatus::InReview => {
                return next == DashboardReservationStatus::Archived;
            }
            DashboardReservationStatus::Resolved => {
                match next {
                    DashboardReservationStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardReservationStatus::Reopened => {
                match next {
                    DashboardReservationStatus::Archived => {
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

    pub fn move_to(&mut self, next: DashboardReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "dashboard reservation", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> DashboardReservation {
        DashboardReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            namespace: self.namespace.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a dashboard reservation in status open.
pub fn new_dashboard_reservation(code: &str, name: &str) -> DashboardReservation {
    DashboardReservation {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        title: "".to_string(),
        checksum: "".to_string(),
        namespace: "".to_string(),
        quantity: 0,
        status: DashboardReservationStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_dashboard_reservation("code-346", "name-692");
        assert_eq!(item.status, DashboardReservationStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_dashboard_reservation("code-346", "name-692");
        if item.move_to(DashboardReservationStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardReservationStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_dashboard_reservation("code-346", "name-692");
        item.status = DashboardReservationStatus::Archived;
        assert!(
            item.move_to(DashboardReservationStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
