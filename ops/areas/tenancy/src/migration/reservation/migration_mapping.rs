use super::*;

use acme_kit::text;

/// Where a migration mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationMappingStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl MigrationMappingStatus {
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

impl std::fmt::Display for MigrationMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a migration mapping and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationMapping {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub retries: i64,
    pub cipher: String,
    pub channel: String,
    pub expires_at: i64,
    pub status: MigrationMappingStatus,
}

impl MigrationMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationMappingStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationMappingStatus) -> bool {
        match self.status {
            MigrationMappingStatus::Planned => {
                return next == MigrationMappingStatus::Scheduled || next == MigrationMappingStatus::Cancelled;
            }
            MigrationMappingStatus::Scheduled => {
                return next == MigrationMappingStatus::Completed || next == MigrationMappingStatus::Cancelled;
            }
            MigrationMappingStatus::InProgress => {
                match next {
                    MigrationMappingStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationMappingStatus::Completed => {
                return next == MigrationMappingStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration mapping {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.retries as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationMapping {
        MigrationMapping {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            retries: self.retries,
            cipher: self.cipher.clone(),
            channel: self.channel.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a migration mapping in status planned.
pub fn new_migration_mapping(code: &str, name: &str) -> MigrationMapping {
    MigrationMapping {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        retries: 0,
        cipher: "".to_string(),
        channel: "".to_string(),
        expires_at: 0,
        status: MigrationMappingStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_migration_mapping("code-95", "name-275");
        assert_eq!(item.status, MigrationMappingStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_migration_mapping("code-95", "name-275");
        if item.move_to(MigrationMappingStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationMappingStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_migration_mapping("code-95", "name-275");
        item.status = MigrationMappingStatus::Cancelled;
        assert!(
            item.move_to(MigrationMappingStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
