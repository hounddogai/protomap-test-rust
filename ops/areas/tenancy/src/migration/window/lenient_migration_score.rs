use super::*;

use acme_kit::text;

/// Where a lenient migration score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientMigrationScoreStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientMigrationScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientMigrationScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient migration score, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationScore {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub notes: String,
    pub rotation_days: i64,
    pub pinned: bool,
    pub rate: f64,
    pub grace_days: i64,
    pub port: i64,
    pub status: LenientMigrationScoreStatus,
}

impl LenientMigrationScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientMigrationScoreStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientMigrationScoreStatus) -> bool {
        match self.status {
            LenientMigrationScoreStatus::Created => {
                match next {
                    LenientMigrationScoreStatus::Validated | LenientMigrationScoreStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientMigrationScoreStatus::Validated => {
                match next {
                    LenientMigrationScoreStatus::Published | LenientMigrationScoreStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientMigrationScoreStatus::Published => {
                return next == LenientMigrationScoreStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientMigrationScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient migration score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "lenient migration score", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.grace_days as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientMigrationScore {
        LenientMigrationScore {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            notes: self.notes.clone(),
            rotation_days: self.rotation_days,
            pinned: self.pinned,
            rate: self.rate,
            grace_days: self.grace_days,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient migration score in status created.
pub fn new_lenient_migration_score(code: &str, label: &str) -> LenientMigrationScore {
    LenientMigrationScore {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        notes: "".to_string(),
        rotation_days: 0,
        pinned: false,
        rate: 0.0,
        grace_days: 0,
        port: 0,
        status: LenientMigrationScoreStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_lenient_migration_score("code-271", "label-457");
        assert_eq!(item.status, LenientMigrationScoreStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_lenient_migration_score("code-271", "label-457");
        if item.move_to(LenientMigrationScoreStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientMigrationScoreStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_lenient_migration_score("code-271", "label-457");
        item.status = LenientMigrationScoreStatus::Retired;
        assert!(
            item.move_to(LenientMigrationScoreStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
