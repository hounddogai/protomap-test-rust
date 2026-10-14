use super::*;

use acme_kit::text;

/// Where a pending migration schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingMigrationScheduleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PendingMigrationScheduleStatus {
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

impl std::fmt::Display for PendingMigrationScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending migration schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingMigrationSchedule {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub burst: i64,
    pub starts_at: i64,
    pub tenant_id: String,
    pub contact_email: String,
    pub status: PendingMigrationScheduleStatus,
}

impl PendingMigrationSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingMigrationScheduleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingMigrationScheduleStatus) -> bool {
        match self.status {
            PendingMigrationScheduleStatus::Idle => {
                return next == PendingMigrationScheduleStatus::Warming
                    || next == PendingMigrationScheduleStatus::Ready;
            }
            PendingMigrationScheduleStatus::Warming => {
                return next == PendingMigrationScheduleStatus::Ready
                    || next == PendingMigrationScheduleStatus::Stopped;
            }
            PendingMigrationScheduleStatus::Ready => {
                match next {
                    PendingMigrationScheduleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingMigrationScheduleStatus::Draining => {
                return next == PendingMigrationScheduleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingMigrationScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending migration schedule {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.starts_at as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingMigrationSchedule {
        PendingMigrationSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            burst: self.burst,
            starts_at: self.starts_at,
            tenant_id: self.tenant_id.clone(),
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending migration schedule in status idle.
pub fn new_pending_migration_schedule(code: &str, name: &str) -> PendingMigrationSchedule {
    PendingMigrationSchedule {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        burst: 0,
        starts_at: 0,
        tenant_id: "".to_string(),
        contact_email: "".to_string(),
        status: PendingMigrationScheduleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_pending_migration_schedule("code-222", "name-65");
        assert_eq!(item.status, PendingMigrationScheduleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_pending_migration_schedule("code-222", "name-65");
        if item.move_to(PendingMigrationScheduleStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingMigrationScheduleStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_pending_migration_schedule("code-222", "name-65");
        item.status = PendingMigrationScheduleStatus::Stopped;
        assert!(
            item.move_to(PendingMigrationScheduleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
