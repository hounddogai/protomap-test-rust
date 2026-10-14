use super::*;

use acme_kit::text;

/// Where a tiered migration adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredMigrationAdjustmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TieredMigrationAdjustmentStatus {
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

impl std::fmt::Display for TieredMigrationAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered migration adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationAdjustment {
    pub code: String,
    pub label: String,
    pub capacity: i64,
    pub issuer: String,
    pub score: f64,
    pub currency: String,
    pub quantity: i64,
    pub timezone: String,
    pub key_id: String,
    pub status: TieredMigrationAdjustmentStatus,
}

impl TieredMigrationAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredMigrationAdjustmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredMigrationAdjustmentStatus) -> bool {
        match self.status {
            TieredMigrationAdjustmentStatus::New => {
                return next == TieredMigrationAdjustmentStatus::Failed;
            }
            TieredMigrationAdjustmentStatus::Queued => {
                return next == TieredMigrationAdjustmentStatus::Running;
            }
            TieredMigrationAdjustmentStatus::Running => {
                return next == TieredMigrationAdjustmentStatus::Failed;
            }
            TieredMigrationAdjustmentStatus::Done => {
                match next {
                    TieredMigrationAdjustmentStatus::Failed => {
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

    pub fn move_to(&mut self, next: TieredMigrationAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered migration adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.capacity as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredMigrationAdjustment {
        TieredMigrationAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            capacity: self.capacity,
            issuer: self.issuer.clone(),
            score: self.score,
            currency: self.currency.clone(),
            quantity: self.quantity,
            timezone: self.timezone.clone(),
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a tiered migration adjustment in status new.
pub fn new_tiered_migration_adjustment(code: &str, label: &str) -> TieredMigrationAdjustment {
    TieredMigrationAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        capacity: 0,
        issuer: "".to_string(),
        score: 0.0,
        currency: "".to_string(),
        quantity: 0,
        timezone: "".to_string(),
        key_id: "".to_string(),
        status: TieredMigrationAdjustmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_tiered_migration_adjustment("code-859", "label-261");
        assert_eq!(item.status, TieredMigrationAdjustmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_tiered_migration_adjustment("code-859", "label-261");
        if item.move_to(TieredMigrationAdjustmentStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredMigrationAdjustmentStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_tiered_migration_adjustment("code-859", "label-261");
        item.status = TieredMigrationAdjustmentStatus::Failed;
        assert!(
            item.move_to(TieredMigrationAdjustmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
