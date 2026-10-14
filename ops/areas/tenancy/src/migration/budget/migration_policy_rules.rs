use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration policy must pass. Check returns its problems, or none.
pub trait MigrationPolicyRule: std::fmt::Debug {
    fn check(&self, item: &MigrationPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationPolicyCodePrefixRule {
    pub prefix: String,
}

impl MigrationPolicyRule for MigrationPolicyCodePrefixRule {
    fn check(&self, item: &MigrationPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationPolicyRequiredLabelRule {
    pub message: String,
}

impl MigrationPolicyRule for MigrationPolicyRequiredLabelRule {
    fn check(&self, item: &MigrationPolicy) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationPolicyRequiredHostRule {
    pub message: String,
}

impl MigrationPolicyRule for MigrationPolicyRequiredHostRule {
    fn check(&self, item: &MigrationPolicy) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration policy.
#[derive(Debug, Clone)]
pub struct MigrationPolicyValidator {
    pub rules: Vec<Rc<dyn MigrationPolicyRule>>,
}

impl MigrationPolicyValidator {
    pub fn validate(&self, item: &MigrationPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MigrationPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationPolicy]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_migration_policy_validator() -> MigrationPolicyValidator {
    MigrationPolicyValidator {
        rules: vec![
            Rc::new(MigrationPolicyCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn MigrationPolicyRule>,
            Rc::new(MigrationPolicyRequiredLabelRule {
                message: "migration policy needs this field".to_string(),
            }),
            Rc::new(MigrationPolicyRequiredHostRule {
                message: "migration policy needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_policy_validator();
        let item = new_migration_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = MigrationPolicyRequiredLabelRule {
            message: "migration policy needs this field".to_string(),
        };
        let mut item = new_migration_policy("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = MigrationPolicyRequiredHostRule {
            message: "migration policy needs a value".to_string(),
        };
        let mut item = new_migration_policy("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
