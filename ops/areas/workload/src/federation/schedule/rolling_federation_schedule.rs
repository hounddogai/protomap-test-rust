use super::*;

use acme_kit::text;

/// Where a rolling federation schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingFederationScheduleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingFederationScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for RollingFederationScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling federation schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationSchedule {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub visible: bool,
    pub revision: i64,
    pub port: i64,
    pub status: RollingFederationScheduleStatus,
}

impl RollingFederationSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingFederationScheduleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingFederationScheduleStatus) -> bool {
        match self.status {
            RollingFederationScheduleStatus::New => {
                return next == RollingFederationScheduleStatus::Queued
                    || next == RollingFederationScheduleStatus::Done;
            }
            RollingFederationScheduleStatus::Queued => {
                match next {
                    RollingFederationScheduleStatus::Done | RollingFederationScheduleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingFederationScheduleStatus::Running => {
                match next {
                    RollingFederationScheduleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingFederationScheduleStatus::Done => {
                return next == RollingFederationScheduleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingFederationScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling federation schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.revision as f64 - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingFederationSchedule {
        RollingFederationSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            visible: self.visible,
            revision: self.revision,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rolling federation schedule in status new.
pub fn new_rolling_federation_schedule(code: &str, name: &str) -> RollingFederationSchedule {
    RollingFederationSchedule {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        visible: false,
        revision: 0,
        port: 0,
        status: RollingFederationScheduleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rolling_federation_schedule("code-858", "name-457");
        assert_eq!(item.status, RollingFederationScheduleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_rolling_federation_schedule("code-858", "name-457");
        if item.move_to(RollingFederationScheduleStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingFederationScheduleStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rolling_federation_schedule("code-858", "name-457");
        item.status = RollingFederationScheduleStatus::Failed;
        assert!(
            item.move_to(RollingFederationScheduleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
