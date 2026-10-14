use super::*;

use acme_kit::text;

/// Where a lenient migration threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientMigrationThresholdStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientMigrationThresholdStatus {
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

impl std::fmt::Display for LenientMigrationThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient migration threshold as the migration team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationThreshold {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub scope: String,
    pub title: String,
    pub currency: String,
    pub margin: f64,
    pub version: i64,
    pub checksum: String,
    pub visible: bool,
    pub created_at: i64,
    pub status: LenientMigrationThresholdStatus,
}

impl LenientMigrationThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientMigrationThresholdStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientMigrationThresholdStatus) -> bool {
        match self.status {
            LenientMigrationThresholdStatus::Draft => {
                return next == LenientMigrationThresholdStatus::Pending;
            }
            LenientMigrationThresholdStatus::Pending => {
                return next == LenientMigrationThresholdStatus::Active
                    || next == LenientMigrationThresholdStatus::Closed;
            }
            LenientMigrationThresholdStatus::Active => {
                match next {
                    LenientMigrationThresholdStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientMigrationThresholdStatus::Suspended => {
                return next == LenientMigrationThresholdStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientMigrationThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient migration threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "lenient migration threshold", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.version as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientMigrationThreshold {
        LenientMigrationThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            scope: self.scope.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            margin: self.margin,
            version: self.version,
            checksum: self.checksum.clone(),
            visible: self.visible,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a lenient migration threshold in status draft.
pub fn new_lenient_migration_threshold(code: &str, label: &str) -> LenientMigrationThreshold {
    LenientMigrationThreshold {
        code: code.to_string(),
        label: label.to_string(),
        reviewer: "".to_string(),
        scope: "".to_string(),
        title: "".to_string(),
        currency: "".to_string(),
        margin: 0.0,
        version: 0,
        checksum: "".to_string(),
        visible: false,
        created_at: 0,
        status: LenientMigrationThresholdStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_lenient_migration_threshold("code-533", "label-349");
        assert_eq!(item.status, LenientMigrationThresholdStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_lenient_migration_threshold("code-533", "label-349");
        if item.move_to(LenientMigrationThresholdStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientMigrationThresholdStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_lenient_migration_threshold("code-533", "label-349");
        item.status = LenientMigrationThresholdStatus::Closed;
        assert!(
            item.move_to(LenientMigrationThresholdStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
