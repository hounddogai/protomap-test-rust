use super::*;

use acme_kit::text;

/// Where a seasonal migration allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalMigrationAllocationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SeasonalMigrationAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for SeasonalMigrationAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal migration allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalMigrationAllocation {
    pub code: String,
    pub title: String,
    pub contact_email: String,
    pub name: String,
    pub timezone: String,
    pub quantity: i64,
    pub status: SeasonalMigrationAllocationStatus,
}

impl SeasonalMigrationAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalMigrationAllocationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalMigrationAllocationStatus) -> bool {
        match self.status {
            SeasonalMigrationAllocationStatus::Idle => {
                match next {
                    SeasonalMigrationAllocationStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalMigrationAllocationStatus::Warming => {
                match next {
                    SeasonalMigrationAllocationStatus::Ready | SeasonalMigrationAllocationStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalMigrationAllocationStatus::Ready => {
                return next == SeasonalMigrationAllocationStatus::Stopped;
            }
            SeasonalMigrationAllocationStatus::Draining => {
                return next == SeasonalMigrationAllocationStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalMigrationAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal migration allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "seasonal migration allocation", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> SeasonalMigrationAllocation {
        SeasonalMigrationAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a seasonal migration allocation in status idle.
pub fn new_seasonal_migration_allocation(code: &str, title: &str) -> SeasonalMigrationAllocation {
    SeasonalMigrationAllocation {
        code: code.to_string(),
        title: title.to_string(),
        contact_email: "".to_string(),
        name: "".to_string(),
        timezone: "".to_string(),
        quantity: 0,
        status: SeasonalMigrationAllocationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_seasonal_migration_allocation("code-972", "title-552");
        assert_eq!(item.status, SeasonalMigrationAllocationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_seasonal_migration_allocation("code-972", "title-552");
        if item.move_to(SeasonalMigrationAllocationStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalMigrationAllocationStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_seasonal_migration_allocation("code-972", "title-552");
        item.status = SeasonalMigrationAllocationStatus::Stopped;
        assert!(
            item.move_to(SeasonalMigrationAllocationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
