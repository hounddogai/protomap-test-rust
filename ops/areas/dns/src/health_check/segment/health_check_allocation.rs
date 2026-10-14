use super::*;

use acme_kit::text;

/// Where a health check allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckAllocationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl HealthCheckAllocationStatus {
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

impl std::fmt::Display for HealthCheckAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A health check allocation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAllocation {
    pub code: String,
    pub name: String,
    pub contact_email: String,
    pub version: i64,
    pub fingerprint: String,
    pub status: HealthCheckAllocationStatus,
}

impl HealthCheckAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckAllocationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckAllocationStatus) -> bool {
        match self.status {
            HealthCheckAllocationStatus::Planned => {
                return next == HealthCheckAllocationStatus::Completed
                    || next == HealthCheckAllocationStatus::Cancelled;
            }
            HealthCheckAllocationStatus::Scheduled => {
                return next == HealthCheckAllocationStatus::InProgress
                    || next == HealthCheckAllocationStatus::Cancelled;
            }
            HealthCheckAllocationStatus::InProgress => {
                match next {
                    HealthCheckAllocationStatus::Completed | HealthCheckAllocationStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckAllocationStatus::Completed => {
                match next {
                    HealthCheckAllocationStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: HealthCheckAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.version as f64
    }

    pub fn copy(&self) -> HealthCheckAllocation {
        HealthCheckAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            contact_email: self.contact_email.clone(),
            version: self.version,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a health check allocation in status planned.
pub fn new_health_check_allocation(code: &str, name: &str) -> HealthCheckAllocation {
    HealthCheckAllocation {
        code: code.to_string(),
        name: name.to_string(),
        contact_email: "".to_string(),
        version: 0,
        fingerprint: "".to_string(),
        status: HealthCheckAllocationStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_health_check_allocation("code-405", "name-402");
        assert_eq!(item.status, HealthCheckAllocationStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_health_check_allocation("code-405", "name-402");
        if item.move_to(HealthCheckAllocationStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckAllocationStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_health_check_allocation("code-405", "name-402");
        item.status = HealthCheckAllocationStatus::Cancelled;
        assert!(
            item.move_to(HealthCheckAllocationStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
