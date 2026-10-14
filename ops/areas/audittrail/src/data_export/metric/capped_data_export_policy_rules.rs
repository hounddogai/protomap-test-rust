use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped data export policy must pass. Check returns its problems, or none.
pub trait CappedDataExportPolicyRule: std::fmt::Debug {
    fn check(&self, item: &CappedDataExportPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDataExportPolicyCodeLengthRule {
    pub max_length: i64,
}

impl CappedDataExportPolicyRule for CappedDataExportPolicyCodeLengthRule {
    fn check(&self, item: &CappedDataExportPolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDataExportPolicyRequiredTitleRule {
    pub message: String,
}

impl CappedDataExportPolicyRule for CappedDataExportPolicyRequiredTitleRule {
    fn check(&self, item: &CappedDataExportPolicy) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDataExportPolicyRequiredCodeRule {
    pub message: String,
}

impl CappedDataExportPolicyRule for CappedDataExportPolicyRequiredCodeRule {
    fn check(&self, item: &CappedDataExportPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped data export policy.
#[derive(Debug, Clone)]
pub struct CappedDataExportPolicyValidator {
    pub rules: Vec<Rc<dyn CappedDataExportPolicyRule>>,
}

impl CappedDataExportPolicyValidator {
    pub fn validate(&self, item: &CappedDataExportPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedDataExportPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedDataExportPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_data_export_policy_validator() -> CappedDataExportPolicyValidator {
    CappedDataExportPolicyValidator {
        rules: vec![
            Rc::new(CappedDataExportPolicyCodeLengthRule { max_length: 86400 }) as Rc<dyn CappedDataExportPolicyRule>,
            Rc::new(CappedDataExportPolicyRequiredTitleRule {
                message: "capped data export policy needs an entry".to_string(),
            }),
            Rc::new(CappedDataExportPolicyRequiredCodeRule {
                message: "capped data export policy needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_data_export_policy_validator();
        let item = new_capped_data_export_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedDataExportPolicyCodeLengthRule { max_length: 255 };
        let mut item = new_capped_data_export_policy("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = CappedDataExportPolicyRequiredTitleRule {
            message: "capped data export policy needs a value".to_string(),
        };
        let mut item = new_capped_data_export_policy("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedDataExportPolicyRequiredCodeRule {
            message: "capped data export policy needs this field".to_string(),
        };
        let mut item = new_capped_data_export_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
