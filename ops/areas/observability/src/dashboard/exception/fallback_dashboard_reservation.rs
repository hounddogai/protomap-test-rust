use super::*;

use acme_kit::text;

/// Where a fallback dashboard reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackDashboardReservationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FallbackDashboardReservationStatus {
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

impl std::fmt::Display for FallbackDashboardReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback dashboard reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDashboardReservation {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub active: bool,
    pub ends_at: i64,
    pub status: FallbackDashboardReservationStatus,
}

impl FallbackDashboardReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackDashboardReservationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackDashboardReservationStatus) -> bool {
        match self.status {
            FallbackDashboardReservationStatus::Planned => {
                match next {
                    FallbackDashboardReservationStatus::Scheduled | FallbackDashboardReservationStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackDashboardReservationStatus::Scheduled => {
                match next {
                    FallbackDashboardReservationStatus::InProgress | FallbackDashboardReservationStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackDashboardReservationStatus::InProgress => {
                match next {
                    FallbackDashboardReservationStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackDashboardReservationStatus::Completed => {
                return next == FallbackDashboardReservationStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackDashboardReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback dashboard reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> FallbackDashboardReservation {
        FallbackDashboardReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            active: self.active,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a fallback dashboard reservation in status planned.
pub fn new_fallback_dashboard_reservation(code: &str, name: &str) -> FallbackDashboardReservation {
    FallbackDashboardReservation {
        code: code.to_string(),
        name: name.to_string(),
        notes: "".to_string(),
        active: false,
        ends_at: 0,
        status: FallbackDashboardReservationStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_fallback_dashboard_reservation("code-825", "name-906");
        assert_eq!(item.status, FallbackDashboardReservationStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_fallback_dashboard_reservation("code-825", "name-906");
        if item.move_to(FallbackDashboardReservationStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackDashboardReservationStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_fallback_dashboard_reservation("code-825", "name-906");
        item.status = FallbackDashboardReservationStatus::Cancelled;
        assert!(
            item.move_to(FallbackDashboardReservationStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
