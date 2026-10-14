use super::*;

use acme_kit::text;

/// Where a regional migration tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalMigrationTierStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RegionalMigrationTierStatus {
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

impl std::fmt::Display for RegionalMigrationTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional migration tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMigrationTier {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub source: String,
    pub host: String,
    pub notes: String,
    pub threshold: f64,
    pub attempts: i64,
    pub quantity: i64,
    pub checksum: String,
    pub version: i64,
    pub status: RegionalMigrationTierStatus,
}

impl RegionalMigrationTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalMigrationTierStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalMigrationTierStatus) -> bool {
        match self.status {
            RegionalMigrationTierStatus::Planned => {
                return next == RegionalMigrationTierStatus::Completed;
            }
            RegionalMigrationTierStatus::Scheduled => {
                match next {
                    RegionalMigrationTierStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalMigrationTierStatus::InProgress => {
                return next == RegionalMigrationTierStatus::Completed
                    || next == RegionalMigrationTierStatus::Cancelled;
            }
            RegionalMigrationTierStatus::Completed => {
                match next {
                    RegionalMigrationTierStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RegionalMigrationTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional migration tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional migration tier", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.threshold - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalMigrationTier {
        RegionalMigrationTier {
            code: self.code.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            source: self.source.clone(),
            host: self.host.clone(),
            notes: self.notes.clone(),
            threshold: self.threshold,
            attempts: self.attempts,
            quantity: self.quantity,
            checksum: self.checksum.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a regional migration tier in status planned.
pub fn new_regional_migration_tier(code: &str, title: &str) -> RegionalMigrationTier {
    RegionalMigrationTier {
        code: code.to_string(),
        title: title.to_string(),
        namespace: "".to_string(),
        source: "".to_string(),
        host: "".to_string(),
        notes: "".to_string(),
        threshold: 0.0,
        attempts: 0,
        quantity: 0,
        checksum: "".to_string(),
        version: 0,
        status: RegionalMigrationTierStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_regional_migration_tier("code-412", "title-823");
        assert_eq!(item.status, RegionalMigrationTierStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_regional_migration_tier("code-412", "title-823");
        if item.move_to(RegionalMigrationTierStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalMigrationTierStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_regional_migration_tier("code-412", "title-823");
        item.status = RegionalMigrationTierStatus::Cancelled;
        assert!(
            item.move_to(RegionalMigrationTierStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
