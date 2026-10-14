use super::*;

use acme_kit::text;

/// Where a migration baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationBaselineStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl MigrationBaselineStatus {
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

impl std::fmt::Display for MigrationBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One migration baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationBaseline {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub audience: String,
    pub notes: String,
    pub external_ref: String,
    pub batch_size: i64,
    pub floor_value: f64,
    pub status: MigrationBaselineStatus,
}

impl MigrationBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationBaselineStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationBaselineStatus) -> bool {
        match self.status {
            MigrationBaselineStatus::Idle => {
                match next {
                    MigrationBaselineStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationBaselineStatus::Warming => {
                match next {
                    MigrationBaselineStatus::Ready | MigrationBaselineStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationBaselineStatus::Ready => {
                return next == MigrationBaselineStatus::Draining;
            }
            MigrationBaselineStatus::Draining => {
                match next {
                    MigrationBaselineStatus::Stopped => {
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

    pub fn move_to(&mut self, next: MigrationBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.floor_value * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationBaseline {
        MigrationBaseline {
            code: self.code.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            audience: self.audience.clone(),
            notes: self.notes.clone(),
            external_ref: self.external_ref.clone(),
            batch_size: self.batch_size,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a migration baseline in status idle.
pub fn new_migration_baseline(code: &str, name: &str) -> MigrationBaseline {
    MigrationBaseline {
        code: code.to_string(),
        name: name.to_string(),
        scope: "".to_string(),
        audience: "".to_string(),
        notes: "".to_string(),
        external_ref: "".to_string(),
        batch_size: 0,
        floor_value: 0.0,
        status: MigrationBaselineStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_migration_baseline("code-291", "name-480");
        assert_eq!(item.status, MigrationBaselineStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_migration_baseline("code-291", "name-480");
        if item.move_to(MigrationBaselineStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationBaselineStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_migration_baseline("code-291", "name-480");
        item.status = MigrationBaselineStatus::Stopped;
        assert!(
            item.move_to(MigrationBaselineStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
