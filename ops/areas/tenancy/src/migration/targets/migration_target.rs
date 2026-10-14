use super::*;

use acme_kit::text;

/// Where a migration target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationTargetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl MigrationTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for MigrationTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A migration target as the migration team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationTarget {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub cipher: String,
    pub audience: String,
    pub pinned: bool,
    pub ttl_seconds: i64,
    pub unit: String,
    pub starts_at: i64,
    pub active: bool,
    pub limit: i64,
    pub status: MigrationTargetStatus,
}

impl MigrationTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationTargetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationTargetStatus) -> bool {
        match self.status {
            MigrationTargetStatus::Created => {
                match next {
                    MigrationTargetStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationTargetStatus::Validated => {
                match next {
                    MigrationTargetStatus::Published | MigrationTargetStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationTargetStatus::Published => {
                return next == MigrationTargetStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration target {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.ttl_seconds as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationTarget {
        MigrationTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            cipher: self.cipher.clone(),
            audience: self.audience.clone(),
            pinned: self.pinned,
            ttl_seconds: self.ttl_seconds,
            unit: self.unit.clone(),
            starts_at: self.starts_at,
            active: self.active,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a migration target in status created.
pub fn new_migration_target(code: &str, name: &str) -> MigrationTarget {
    MigrationTarget {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        cipher: "".to_string(),
        audience: "".to_string(),
        pinned: false,
        ttl_seconds: 0,
        unit: "".to_string(),
        starts_at: 0,
        active: false,
        limit: 0,
        status: MigrationTargetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_migration_target("code-648", "name-471");
        assert_eq!(item.status, MigrationTargetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_migration_target("code-648", "name-471");
        if item.move_to(MigrationTargetStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationTargetStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_migration_target("code-648", "name-471");
        item.status = MigrationTargetStatus::Retired;
        assert!(
            item.move_to(MigrationTargetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
