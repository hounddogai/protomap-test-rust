use super::*;

use acme_kit::text;

/// Where a manual migration estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualMigrationEstimateStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ManualMigrationEstimateStatus {
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

impl std::fmt::Display for ManualMigrationEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual migration estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualMigrationEstimate {
    pub code: String,
    pub label: String,
    pub weight: f64,
    pub principal: String,
    pub quantity: i64,
    pub summary: String,
    pub margin: f64,
    pub budget: f64,
    pub region: String,
    pub contact_email: String,
    pub status: ManualMigrationEstimateStatus,
}

impl ManualMigrationEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualMigrationEstimateStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualMigrationEstimateStatus) -> bool {
        match self.status {
            ManualMigrationEstimateStatus::Draft => {
                return next == ManualMigrationEstimateStatus::Active
                    || next == ManualMigrationEstimateStatus::Suspended;
            }
            ManualMigrationEstimateStatus::Pending => {
                match next {
                    ManualMigrationEstimateStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualMigrationEstimateStatus::Active => {
                return next == ManualMigrationEstimateStatus::Closed;
            }
            ManualMigrationEstimateStatus::Suspended => {
                match next {
                    ManualMigrationEstimateStatus::Closed => {
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

    pub fn move_to(&mut self, next: ManualMigrationEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual migration estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.quantity as f64) * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualMigrationEstimate {
        ManualMigrationEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            weight: self.weight,
            principal: self.principal.clone(),
            quantity: self.quantity,
            summary: self.summary.clone(),
            margin: self.margin,
            budget: self.budget,
            region: self.region.clone(),
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a manual migration estimate in status draft.
pub fn new_manual_migration_estimate(code: &str, label: &str) -> ManualMigrationEstimate {
    ManualMigrationEstimate {
        code: code.to_string(),
        label: label.to_string(),
        weight: 0.0,
        principal: "".to_string(),
        quantity: 0,
        summary: "".to_string(),
        margin: 0.0,
        budget: 0.0,
        region: "".to_string(),
        contact_email: "".to_string(),
        status: ManualMigrationEstimateStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_manual_migration_estimate("code-928", "label-15");
        assert_eq!(item.status, ManualMigrationEstimateStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_manual_migration_estimate("code-928", "label-15");
        if item.move_to(ManualMigrationEstimateStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualMigrationEstimateStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_manual_migration_estimate("code-928", "label-15");
        item.status = ManualMigrationEstimateStatus::Closed;
        assert!(
            item.move_to(ManualMigrationEstimateStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
