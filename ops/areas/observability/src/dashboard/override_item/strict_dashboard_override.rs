use super::*;

use acme_kit::text;

/// Where a strict dashboard override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictDashboardOverrideStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl StrictDashboardOverrideStatus {
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

impl std::fmt::Display for StrictDashboardOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict dashboard override as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardOverride {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub description: String,
    pub host: String,
    pub region: String,
    pub batch_size: i64,
    pub audience: String,
    pub attempts: i64,
    pub contact_email: String,
    pub timezone: String,
    pub status: StrictDashboardOverrideStatus,
}

impl StrictDashboardOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictDashboardOverrideStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictDashboardOverrideStatus) -> bool {
        match self.status {
            StrictDashboardOverrideStatus::Planned => {
                return next == StrictDashboardOverrideStatus::Scheduled
                    || next == StrictDashboardOverrideStatus::Cancelled;
            }
            StrictDashboardOverrideStatus::Scheduled => {
                match next {
                    StrictDashboardOverrideStatus::Completed | StrictDashboardOverrideStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictDashboardOverrideStatus::InProgress => {
                match next {
                    StrictDashboardOverrideStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictDashboardOverrideStatus::Completed => {
                return next == StrictDashboardOverrideStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictDashboardOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict dashboard override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict dashboard override", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.batch_size as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictDashboardOverride {
        StrictDashboardOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            segment: self.segment.clone(),
            description: self.description.clone(),
            host: self.host.clone(),
            region: self.region.clone(),
            batch_size: self.batch_size,
            audience: self.audience.clone(),
            attempts: self.attempts,
            contact_email: self.contact_email.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a strict dashboard override in status planned.
pub fn new_strict_dashboard_override(code: &str, name: &str) -> StrictDashboardOverride {
    StrictDashboardOverride {
        code: code.to_string(),
        name: name.to_string(),
        segment: "".to_string(),
        description: "".to_string(),
        host: "".to_string(),
        region: "".to_string(),
        batch_size: 0,
        audience: "".to_string(),
        attempts: 0,
        contact_email: "".to_string(),
        timezone: "".to_string(),
        status: StrictDashboardOverrideStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_strict_dashboard_override("code-715", "name-73");
        assert_eq!(item.status, StrictDashboardOverrideStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_strict_dashboard_override("code-715", "name-73");
        if item.move_to(StrictDashboardOverrideStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictDashboardOverrideStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_strict_dashboard_override("code-715", "name-73");
        item.status = StrictDashboardOverrideStatus::Cancelled;
        assert!(
            item.move_to(StrictDashboardOverrideStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
