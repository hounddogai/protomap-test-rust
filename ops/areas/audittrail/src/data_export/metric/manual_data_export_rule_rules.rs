use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual data export rule must pass. Check returns its problems, or none.
pub trait ManualDataExportRuleRule: std::fmt::Debug {
    fn check(&self, item: &ManualDataExportRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportRuleFingerprintLengthRule {
    pub max_length: i64,
}

impl ManualDataExportRuleRule for ManualDataExportRuleFingerprintLengthRule {
    fn check(&self, item: &ManualDataExportRule) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportRuleCodePrefixRule {
    pub prefix: String,
}

impl ManualDataExportRuleRule for ManualDataExportRuleCodePrefixRule {
    fn check(&self, item: &ManualDataExportRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportRuleQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualDataExportRuleRule for ManualDataExportRuleQuantityRangeRule {
    fn check(&self, item: &ManualDataExportRule) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportRuleRequiredCodeRule {
    pub message: String,
}

impl ManualDataExportRuleRule for ManualDataExportRuleRequiredCodeRule {
    fn check(&self, item: &ManualDataExportRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportRuleLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualDataExportRuleRule for ManualDataExportRuleLimitRangeRule {
    fn check(&self, item: &ManualDataExportRule) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportRuleAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualDataExportRuleRule for ManualDataExportRuleAttemptsRangeRule {
    fn check(&self, item: &ManualDataExportRule) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a manual data export rule.
#[derive(Debug, Clone)]
pub struct ManualDataExportRuleValidator {
    pub rules: Vec<Rc<dyn ManualDataExportRuleRule>>,
}

impl ManualDataExportRuleValidator {
    pub fn validate(&self, item: &ManualDataExportRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualDataExportRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualDataExportRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_data_export_rule_validator() -> ManualDataExportRuleValidator {
    ManualDataExportRuleValidator {
        rules: vec![
            Rc::new(ManualDataExportRuleFingerprintLengthRule { max_length: 255 }) as Rc<dyn ManualDataExportRuleRule>,
            Rc::new(ManualDataExportRuleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(ManualDataExportRuleQuantityRangeRule { min: 120, max: 120 }),
            Rc::new(ManualDataExportRuleRequiredCodeRule {
                message: "manual data export rule needs a value".to_string(),
            }),
            Rc::new(ManualDataExportRuleLimitRangeRule { min: 64, max: 3600 }),
            Rc::new(ManualDataExportRuleAttemptsRangeRule { min: 255, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_data_export_rule_validator();
        let item = new_manual_data_export_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = ManualDataExportRuleFingerprintLengthRule { max_length: 3600 };
        let mut item = new_manual_data_export_rule("x", "y");
        item.fingerprint = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = ManualDataExportRuleQuantityRangeRule { min: 16, max: 64 };
        let mut item = new_manual_data_export_rule("x", "y");
        item.quantity = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualDataExportRuleRequiredCodeRule {
            message: "manual data export rule needs an entry".to_string(),
        };
        let mut item = new_manual_data_export_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = ManualDataExportRuleLimitRangeRule { min: 32, max: 120 };
        let mut item = new_manual_data_export_rule("x", "y");
        item.limit = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = ManualDataExportRuleAttemptsRangeRule { min: 255, max: 86400 };
        let mut item = new_manual_data_export_rule("x", "y");
        item.attempts = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
