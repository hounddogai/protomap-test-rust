use super::*;

use acme_kit::text;

/// Where a health check cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckCycleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl HealthCheckCycleStatus {
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

impl std::fmt::Display for HealthCheckCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A health check cycle as the health check team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckCycle {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub contact_email: String,
    pub capacity: i64,
    pub grace_days: i64,
    pub window_days: i64,
    pub cost: f64,
    pub maximum: f64,
    pub status: HealthCheckCycleStatus,
}

impl HealthCheckCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckCycleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckCycleStatus) -> bool {
        match self.status {
            HealthCheckCycleStatus::Planned => {
                return next == HealthCheckCycleStatus::Scheduled;
            }
            HealthCheckCycleStatus::Scheduled => {
                match next {
                    HealthCheckCycleStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckCycleStatus::InProgress => {
                match next {
                    HealthCheckCycleStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckCycleStatus::Completed => {
                return next == HealthCheckCycleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: HealthCheckCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.grace_days as f64 - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckCycle {
        HealthCheckCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            contact_email: self.contact_email.clone(),
            capacity: self.capacity,
            grace_days: self.grace_days,
            window_days: self.window_days,
            cost: self.cost,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a health check cycle in status planned.
pub fn new_health_check_cycle(code: &str, label: &str) -> HealthCheckCycle {
    HealthCheckCycle {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        contact_email: "".to_string(),
        capacity: 0,
        grace_days: 0,
        window_days: 0,
        cost: 0.0,
        maximum: 0.0,
        status: HealthCheckCycleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_health_check_cycle("code-302", "label-61");
        assert_eq!(item.status, HealthCheckCycleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_health_check_cycle("code-302", "label-61");
        if item.move_to(HealthCheckCycleStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckCycleStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_health_check_cycle("code-302", "label-61");
        item.status = HealthCheckCycleStatus::Cancelled;
        assert!(
            item.move_to(HealthCheckCycleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
