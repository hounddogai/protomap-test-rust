use super::*;

use acme_kit::text;

/// Where a legacy migration adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyMigrationAdjustmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LegacyMigrationAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for LegacyMigrationAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy migration adjustment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMigrationAdjustment {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub category: String,
    pub cost: f64,
    pub audience: String,
    pub owner_id: String,
    pub status: LegacyMigrationAdjustmentStatus,
}

impl LegacyMigrationAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyMigrationAdjustmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyMigrationAdjustmentStatus) -> bool {
        match self.status {
            LegacyMigrationAdjustmentStatus::Draft => {
                return next == LegacyMigrationAdjustmentStatus::Pending
                    || next == LegacyMigrationAdjustmentStatus::Closed;
            }
            LegacyMigrationAdjustmentStatus::Pending => {
                match next {
                    LegacyMigrationAdjustmentStatus::Active | LegacyMigrationAdjustmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyMigrationAdjustmentStatus::Active => {
                return next == LegacyMigrationAdjustmentStatus::Closed;
            }
            LegacyMigrationAdjustmentStatus::Suspended => {
                match next {
                    LegacyMigrationAdjustmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: LegacyMigrationAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy migration adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.cost
    }

    pub fn copy(&self) -> LegacyMigrationAdjustment {
        LegacyMigrationAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            category: self.category.clone(),
            cost: self.cost,
            audience: self.audience.clone(),
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a legacy migration adjustment in status draft.
pub fn new_legacy_migration_adjustment(code: &str, title: &str) -> LegacyMigrationAdjustment {
    LegacyMigrationAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        category: "".to_string(),
        cost: 0.0,
        audience: "".to_string(),
        owner_id: "".to_string(),
        status: LegacyMigrationAdjustmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_legacy_migration_adjustment("code-272", "title-532");
        assert_eq!(item.status, LegacyMigrationAdjustmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_legacy_migration_adjustment("code-272", "title-532");
        if item.move_to(LegacyMigrationAdjustmentStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyMigrationAdjustmentStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_legacy_migration_adjustment("code-272", "title-532");
        item.status = LegacyMigrationAdjustmentStatus::Closed;
        assert!(
            item.move_to(LegacyMigrationAdjustmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
