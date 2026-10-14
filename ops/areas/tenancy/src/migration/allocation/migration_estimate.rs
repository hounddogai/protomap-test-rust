use super::*;

use acme_kit::text;

/// Where a migration estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl MigrationEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for MigrationEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A migration estimate as the migration team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationEstimate {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub description: String,
    pub label: String,
    pub status: MigrationEstimateStatus,
}

impl MigrationEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationEstimateStatus) -> bool {
        match self.status {
            MigrationEstimateStatus::Created => {
                match next {
                    MigrationEstimateStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationEstimateStatus::Validated => {
                return next == MigrationEstimateStatus::Published || next == MigrationEstimateStatus::Retired;
            }
            MigrationEstimateStatus::Published => {
                return next == MigrationEstimateStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.burst as f64
    }

    pub fn copy(&self) -> MigrationEstimate {
        MigrationEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            description: self.description.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a migration estimate in status created.
pub fn new_migration_estimate(code: &str, title: &str) -> MigrationEstimate {
    MigrationEstimate {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        description: "".to_string(),
        label: "".to_string(),
        status: MigrationEstimateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_migration_estimate("code-201", "title-45");
        assert_eq!(item.status, MigrationEstimateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_migration_estimate("code-201", "title-45");
        if item.move_to(MigrationEstimateStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationEstimateStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_migration_estimate("code-201", "title-45");
        item.status = MigrationEstimateStatus::Retired;
        assert!(
            item.move_to(MigrationEstimateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
