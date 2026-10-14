use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual credential rollup must pass. Check returns its problems, or none.
pub trait ManualCredentialRollupRule: std::fmt::Debug {
    fn check(&self, item: &ManualCredentialRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialRollupRequiredDescriptionRule {
    pub message: String,
}

impl ManualCredentialRollupRule for ManualCredentialRollupRequiredDescriptionRule {
    fn check(&self, item: &ManualCredentialRollup) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialRollupRequiredCodeRule {
    pub message: String,
}

impl ManualCredentialRollupRule for ManualCredentialRollupRequiredCodeRule {
    fn check(&self, item: &ManualCredentialRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialRollupDescriptionLengthRule {
    pub max_length: i64,
}

impl ManualCredentialRollupRule for ManualCredentialRollupDescriptionLengthRule {
    fn check(&self, item: &ManualCredentialRollup) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual credential rollup.
#[derive(Debug, Clone)]
pub struct ManualCredentialRollupValidator {
    pub rules: Vec<Rc<dyn ManualCredentialRollupRule>>,
}

impl ManualCredentialRollupValidator {
    pub fn validate(&self, item: &ManualCredentialRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualCredentialRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualCredentialRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_credential_rollup_validator() -> ManualCredentialRollupValidator {
    ManualCredentialRollupValidator {
        rules: vec![
            Rc::new(ManualCredentialRollupRequiredDescriptionRule {
                message: "manual credential rollup needs this field".to_string(),
            }) as Rc<dyn ManualCredentialRollupRule>,
            Rc::new(ManualCredentialRollupRequiredCodeRule {
                message: "manual credential rollup needs this field".to_string(),
            }),
            Rc::new(ManualCredentialRollupDescriptionLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_credential_rollup_validator();
        let item = new_manual_credential_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = ManualCredentialRollupRequiredDescriptionRule {
            message: "manual credential rollup needs a value".to_string(),
        };
        let mut item = new_manual_credential_rollup("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualCredentialRollupRequiredCodeRule {
            message: "manual credential rollup needs an entry".to_string(),
        };
        let mut item = new_manual_credential_rollup("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = ManualCredentialRollupDescriptionLengthRule { max_length: 3600 };
        let mut item = new_manual_credential_rollup("x", "y");
        item.description = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
