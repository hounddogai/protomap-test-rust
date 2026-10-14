use super::*;

use acme_kit::text;

/// Where a migration adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationAdjustmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl MigrationAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for MigrationAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a migration adjustment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationAdjustment {
    pub code: String,
    pub title: String,
    pub source: String,
    pub attempts: i64,
    pub limit: i64,
    pub host: String,
    pub summary: String,
    pub status: MigrationAdjustmentStatus,
}

impl MigrationAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationAdjustmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationAdjustmentStatus) -> bool {
        match self.status {
            MigrationAdjustmentStatus::Created => {
                match next {
                    MigrationAdjustmentStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationAdjustmentStatus::Validated => {
                return next == MigrationAdjustmentStatus::Published || next == MigrationAdjustmentStatus::Retired;
            }
            MigrationAdjustmentStatus::Published => {
                match next {
                    MigrationAdjustmentStatus::Retired => {
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

    pub fn move_to(&mut self, next: MigrationAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.limit as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationAdjustment {
        MigrationAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            attempts: self.attempts,
            limit: self.limit,
            host: self.host.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a migration adjustment in status created.
pub fn new_migration_adjustment(code: &str, title: &str) -> MigrationAdjustment {
    MigrationAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        attempts: 0,
        limit: 0,
        host: "".to_string(),
        summary: "".to_string(),
        status: MigrationAdjustmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_migration_adjustment("code-112", "title-953");
        assert_eq!(item.status, MigrationAdjustmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_migration_adjustment("code-112", "title-953");
        if item.move_to(MigrationAdjustmentStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationAdjustmentStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_migration_adjustment("code-112", "title-953");
        item.status = MigrationAdjustmentStatus::Retired;
        assert!(
            item.move_to(MigrationAdjustmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
