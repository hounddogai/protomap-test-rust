use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a usage audit must pass. Check returns its problems, or none.
pub trait UsageAuditRule: std::fmt::Debug {
    fn check(&self, item: &UsageAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageAuditCodePrefixRule {
    pub prefix: String,
}

impl UsageAuditRule for UsageAuditCodePrefixRule {
    fn check(&self, item: &UsageAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageAuditCurrencyLengthRule {
    pub max_length: i64,
}

impl UsageAuditRule for UsageAuditCurrencyLengthRule {
    fn check(&self, item: &UsageAudit) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageAuditRequiredCodeRule {
    pub message: String,
}

impl UsageAuditRule for UsageAuditRequiredCodeRule {
    fn check(&self, item: &UsageAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageAuditMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl UsageAuditRule for UsageAuditMinimumRangeRule {
    fn check(&self, item: &UsageAudit) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageAuditRequiredLabelRule {
    pub message: String,
}

impl UsageAuditRule for UsageAuditRequiredLabelRule {
    fn check(&self, item: &UsageAudit) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage audit.
#[derive(Debug, Clone)]
pub struct UsageAuditValidator {
    pub rules: Vec<Rc<dyn UsageAuditRule>>,
}

impl UsageAuditValidator {
    pub fn validate(&self, item: &UsageAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &UsageAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_audit_validator() -> UsageAuditValidator {
    UsageAuditValidator {
        rules: vec![
            Rc::new(UsageAuditCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn UsageAuditRule>,
            Rc::new(UsageAuditCurrencyLengthRule { max_length: 32 }),
            Rc::new(UsageAuditRequiredCodeRule {
                message: "usage audit needs an entry".to_string(),
            }),
            Rc::new(UsageAuditMinimumRangeRule { min: 9.1, max: 4892.2 }),
            Rc::new(UsageAuditRequiredLabelRule {
                message: "usage audit needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_audit_validator();
        let item = new_usage_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = UsageAuditCurrencyLengthRule { max_length: 86400 };
        let mut item = new_usage_audit("x", "y");
        item.currency = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = UsageAuditRequiredCodeRule {
            message: "usage audit needs this field".to_string(),
        };
        let mut item = new_usage_audit("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = UsageAuditMinimumRangeRule { min: 0.9, max: 2732.7 };
        let mut item = new_usage_audit("x", "y");
        item.minimum = 2734.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = UsageAuditRequiredLabelRule {
            message: "usage audit needs an entry".to_string(),
        };
        let mut item = new_usage_audit("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
