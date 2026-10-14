use super::*;

use acme_kit::text;

/// Where a global migration rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalMigrationRuleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl GlobalMigrationRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for GlobalMigrationRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global migration rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMigrationRule {
    pub code: String,
    pub label: String,
    pub description: String,
    pub audience: String,
    pub ttl_seconds: i64,
    pub external_ref: String,
    pub issuer: String,
    pub status: GlobalMigrationRuleStatus,
}

impl GlobalMigrationRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalMigrationRuleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalMigrationRuleStatus) -> bool {
        match self.status {
            GlobalMigrationRuleStatus::Proposed => {
                match next {
                    GlobalMigrationRuleStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalMigrationRuleStatus::Approved => {
                return next == GlobalMigrationRuleStatus::Rejected;
            }
            GlobalMigrationRuleStatus::Rejected => {
                match next {
                    GlobalMigrationRuleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: GlobalMigrationRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global migration rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.ttl_seconds as f64
    }

    pub fn copy(&self) -> GlobalMigrationRule {
        GlobalMigrationRule {
            code: self.code.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            audience: self.audience.clone(),
            ttl_seconds: self.ttl_seconds,
            external_ref: self.external_ref.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global migration rule in status proposed.
pub fn new_global_migration_rule(code: &str, label: &str) -> GlobalMigrationRule {
    GlobalMigrationRule {
        code: code.to_string(),
        label: label.to_string(),
        description: "".to_string(),
        audience: "".to_string(),
        ttl_seconds: 0,
        external_ref: "".to_string(),
        issuer: "".to_string(),
        status: GlobalMigrationRuleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_global_migration_rule("code-302", "label-85");
        assert_eq!(item.status, GlobalMigrationRuleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_global_migration_rule("code-302", "label-85");
        if item.move_to(GlobalMigrationRuleStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalMigrationRuleStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_global_migration_rule("code-302", "label-85");
        item.status = GlobalMigrationRuleStatus::Withdrawn;
        assert!(
            item.move_to(GlobalMigrationRuleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
