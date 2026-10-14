use super::*;

use acme_kit::text;

/// Where a regional dashboard reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalDashboardReservationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionalDashboardReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RegionalDashboardReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional dashboard reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDashboardReservation {
    pub code: String,
    pub title: String,
    pub capacity: i64,
    pub port: i64,
    pub ceiling: f64,
    pub region: String,
    pub archived: bool,
    pub duration_minutes: i64,
    pub ends_at: i64,
    pub reviewer: String,
    pub margin: f64,
    pub status: RegionalDashboardReservationStatus,
}

impl RegionalDashboardReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalDashboardReservationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalDashboardReservationStatus) -> bool {
        match self.status {
            RegionalDashboardReservationStatus::Draft => {
                return next == RegionalDashboardReservationStatus::Suspended;
            }
            RegionalDashboardReservationStatus::Pending => {
                return next == RegionalDashboardReservationStatus::Active
                    || next == RegionalDashboardReservationStatus::Closed;
            }
            RegionalDashboardReservationStatus::Active => {
                match next {
                    RegionalDashboardReservationStatus::Suspended | RegionalDashboardReservationStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalDashboardReservationStatus::Suspended => {
                match next {
                    RegionalDashboardReservationStatus::Closed => {
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

    pub fn move_to(&mut self, next: RegionalDashboardReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional dashboard reservation {} cannot move from {} to {}",
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
        format!("{} {}", "regional dashboard reservation", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.port as f64 + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalDashboardReservation {
        RegionalDashboardReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            capacity: self.capacity,
            port: self.port,
            ceiling: self.ceiling,
            region: self.region.clone(),
            archived: self.archived,
            duration_minutes: self.duration_minutes,
            ends_at: self.ends_at,
            reviewer: self.reviewer.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a regional dashboard reservation in status draft.
pub fn new_regional_dashboard_reservation(code: &str, title: &str) -> RegionalDashboardReservation {
    RegionalDashboardReservation {
        code: code.to_string(),
        title: title.to_string(),
        capacity: 0,
        port: 0,
        ceiling: 0.0,
        region: "".to_string(),
        archived: false,
        duration_minutes: 0,
        ends_at: 0,
        reviewer: "".to_string(),
        margin: 0.0,
        status: RegionalDashboardReservationStatus::Draft,
    }
}
