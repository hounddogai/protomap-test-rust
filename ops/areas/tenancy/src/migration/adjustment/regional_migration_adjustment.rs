use super::*;

use acme_kit::text;

/// Where a regional migration adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalMigrationAdjustmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionalMigrationAdjustmentStatus {
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

impl std::fmt::Display for RegionalMigrationAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional migration adjustment as the migration team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMigrationAdjustment {
    pub code: String,
    pub name: String,
    pub discount: f64,
    pub notes: String,
    pub floor_value: f64,
    pub source: String,
    pub attempts: i64,
    pub threshold: f64,
    pub status: RegionalMigrationAdjustmentStatus,
}

impl RegionalMigrationAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalMigrationAdjustmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalMigrationAdjustmentStatus) -> bool {
        match self.status {
            RegionalMigrationAdjustmentStatus::Draft => {
                return next == RegionalMigrationAdjustmentStatus::Closed;
            }
            RegionalMigrationAdjustmentStatus::Pending => {
                match next {
                    RegionalMigrationAdjustmentStatus::Suspended | RegionalMigrationAdjustmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalMigrationAdjustmentStatus::Active => {
                return next == RegionalMigrationAdjustmentStatus::Suspended
                    || next == RegionalMigrationAdjustmentStatus::Closed;
            }
            RegionalMigrationAdjustmentStatus::Suspended => {
                match next {
                    RegionalMigrationAdjustmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: RegionalMigrationAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional migration adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.threshold + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalMigrationAdjustment {
        RegionalMigrationAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            discount: self.discount,
            notes: self.notes.clone(),
            floor_value: self.floor_value,
            source: self.source.clone(),
            attempts: self.attempts,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a regional migration adjustment in status draft.
pub fn new_regional_migration_adjustment(code: &str, name: &str) -> RegionalMigrationAdjustment {
    RegionalMigrationAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        discount: 0.0,
        notes: "".to_string(),
        floor_value: 0.0,
        source: "".to_string(),
        attempts: 0,
        threshold: 0.0,
        status: RegionalMigrationAdjustmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_regional_migration_adjustment("code-493", "name-259");
        assert_eq!(item.status, RegionalMigrationAdjustmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_regional_migration_adjustment("code-493", "name-259");
        if item.move_to(RegionalMigrationAdjustmentStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalMigrationAdjustmentStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_regional_migration_adjustment("code-493", "name-259");
        item.status = RegionalMigrationAdjustmentStatus::Closed;
        assert!(
            item.move_to(RegionalMigrationAdjustmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
