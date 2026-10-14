use super::*;

use acme_kit::text;

/// Where a capped dashboard exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedDashboardExceptionStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl CappedDashboardExceptionStatus {
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

impl std::fmt::Display for CappedDashboardExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped dashboard exception as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardException {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub starts_at: i64,
    pub checksum: String,
    pub country: String,
    pub duration_minutes: i64,
    pub name: String,
    pub external_ref: String,
    pub status: CappedDashboardExceptionStatus,
}

impl CappedDashboardException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedDashboardExceptionStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedDashboardExceptionStatus) -> bool {
        match self.status {
            CappedDashboardExceptionStatus::Planned => {
                match next {
                    CappedDashboardExceptionStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedDashboardExceptionStatus::Scheduled => {
                match next {
                    CappedDashboardExceptionStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedDashboardExceptionStatus::InProgress => {
                return next == CappedDashboardExceptionStatus::Cancelled;
            }
            CappedDashboardExceptionStatus::Completed => {
                match next {
                    CappedDashboardExceptionStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: CappedDashboardExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped dashboard exception {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        (self.ceiling * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedDashboardException {
        CappedDashboardException {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            starts_at: self.starts_at,
            checksum: self.checksum.clone(),
            country: self.country.clone(),
            duration_minutes: self.duration_minutes,
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a capped dashboard exception in status planned.
pub fn new_capped_dashboard_exception(code: &str, label: &str) -> CappedDashboardException {
    CappedDashboardException {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        starts_at: 0,
        checksum: "".to_string(),
        country: "".to_string(),
        duration_minutes: 0,
        name: "".to_string(),
        external_ref: "".to_string(),
        status: CappedDashboardExceptionStatus::Planned,
    }
}
