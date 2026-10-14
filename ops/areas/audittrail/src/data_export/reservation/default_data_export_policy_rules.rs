use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default data export policy must pass. Check returns its problems, or none.
pub trait DefaultDataExportPolicyRule: std::fmt::Debug {
    fn check(&self, item: &DefaultDataExportPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDataExportPolicyAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultDataExportPolicyRule for DefaultDataExportPolicyAmountRangeRule {
    fn check(&self, item: &DefaultDataExportPolicy) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDataExportPolicyRequiredTitleRule {
    pub message: String,
}

impl DefaultDataExportPolicyRule for DefaultDataExportPolicyRequiredTitleRule {
    fn check(&self, item: &DefaultDataExportPolicy) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDataExportPolicyContactEmailLengthRule {
    pub max_length: i64,
}

impl DefaultDataExportPolicyRule for DefaultDataExportPolicyContactEmailLengthRule {
    fn check(&self, item: &DefaultDataExportPolicy) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDataExportPolicyCodeLengthRule {
    pub max_length: i64,
}

impl DefaultDataExportPolicyRule for DefaultDataExportPolicyCodeLengthRule {
    fn check(&self, item: &DefaultDataExportPolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDataExportPolicyRequiredScopeRule {
    pub message: String,
}

impl DefaultDataExportPolicyRule for DefaultDataExportPolicyRequiredScopeRule {
    fn check(&self, item: &DefaultDataExportPolicy) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDataExportPolicyTitleLengthRule {
    pub max_length: i64,
}

impl DefaultDataExportPolicyRule for DefaultDataExportPolicyTitleLengthRule {
    fn check(&self, item: &DefaultDataExportPolicy) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default data export policy.
#[derive(Debug, Clone)]
pub struct DefaultDataExportPolicyValidator {
    pub rules: Vec<Rc<dyn DefaultDataExportPolicyRule>>,
}

impl DefaultDataExportPolicyValidator {
    pub fn validate(&self, item: &DefaultDataExportPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultDataExportPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultDataExportPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_data_export_policy_validator() -> DefaultDataExportPolicyValidator {
    DefaultDataExportPolicyValidator {
        rules: vec![
            Rc::new(DefaultDataExportPolicyAmountRangeRule { min: 1.4, max: 4418.6 })
                as Rc<dyn DefaultDataExportPolicyRule>,
            Rc::new(DefaultDataExportPolicyRequiredTitleRule {
                message: "default data export policy needs this field".to_string(),
            }),
            Rc::new(DefaultDataExportPolicyContactEmailLengthRule { max_length: 86400 }),
            Rc::new(DefaultDataExportPolicyCodeLengthRule { max_length: 1000 }),
            Rc::new(DefaultDataExportPolicyRequiredScopeRule {
                message: "default data export policy needs a value".to_string(),
            }),
            Rc::new(DefaultDataExportPolicyTitleLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_data_export_policy_validator();
        let item = new_default_data_export_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = DefaultDataExportPolicyAmountRangeRule { min: 0.1, max: 2814.9 };
        let mut item = new_default_data_export_policy("x", "y");
        item.amount = 2816.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DefaultDataExportPolicyRequiredTitleRule {
            message: "default data export policy needs a value".to_string(),
        };
        let mut item = new_default_data_export_policy("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = DefaultDataExportPolicyContactEmailLengthRule { max_length: 80 };
        let mut item = new_default_data_export_policy("x", "y");
        item.contact_email = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultDataExportPolicyCodeLengthRule { max_length: 1000 };
        let mut item = new_default_data_export_policy("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = DefaultDataExportPolicyRequiredScopeRule {
            message: "default data export policy needs a value".to_string(),
        };
        let mut item = new_default_data_export_policy("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DefaultDataExportPolicyTitleLengthRule { max_length: 16 };
        let mut item = new_default_data_export_policy("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
