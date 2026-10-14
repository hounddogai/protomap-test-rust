use super::*;

use acme_kit::text;

/// Where a strict health check allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictHealthCheckAllocationStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl StrictHealthCheckAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for StrictHealthCheckAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict health check allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckAllocation {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub key_id: String,
    pub created_at: i64,
    pub attempts: i64,
    pub capacity: i64,
    pub starts_at: i64,
    pub status: StrictHealthCheckAllocationStatus,
}

impl StrictHealthCheckAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictHealthCheckAllocationStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictHealthCheckAllocationStatus) -> bool {
        match self.status {
            StrictHealthCheckAllocationStatus::Created => {
                return next == StrictHealthCheckAllocationStatus::Validated;
            }
            StrictHealthCheckAllocationStatus::Validated => {
                match next {
                    StrictHealthCheckAllocationStatus::Published | StrictHealthCheckAllocationStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictHealthCheckAllocationStatus::Published => {
                return next == StrictHealthCheckAllocationStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictHealthCheckAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict health check allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict health check allocation", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.attempts as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictHealthCheckAllocation {
        StrictHealthCheckAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            key_id: self.key_id.clone(),
            created_at: self.created_at,
            attempts: self.attempts,
            capacity: self.capacity,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a strict health check allocation in status created.
pub fn new_strict_health_check_allocation(code: &str, title: &str) -> StrictHealthCheckAllocation {
    StrictHealthCheckAllocation {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        key_id: "".to_string(),
        created_at: 0,
        attempts: 0,
        capacity: 0,
        starts_at: 0,
        status: StrictHealthCheckAllocationStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_strict_health_check_allocation("code-80", "title-94");
        assert_eq!(item.status, StrictHealthCheckAllocationStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_strict_health_check_allocation("code-80", "title-94");
        if item.move_to(StrictHealthCheckAllocationStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictHealthCheckAllocationStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_strict_health_check_allocation("code-80", "title-94");
        item.status = StrictHealthCheckAllocationStatus::Retired;
        assert!(
            item.move_to(StrictHealthCheckAllocationStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
