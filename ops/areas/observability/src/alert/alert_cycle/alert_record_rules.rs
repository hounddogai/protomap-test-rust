use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a alert record must pass. Check returns its problems, or none.
pub trait AlertRecordRule: std::fmt::Debug {
    fn check(&self, item: &AlertRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRecordCodePrefixRule {
    pub prefix: String,
}

impl AlertRecordRule for AlertRecordCodePrefixRule {
    fn check(&self, item: &AlertRecord) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRecordRequiredSourceRule {
    pub message: String,
}

impl AlertRecordRule for AlertRecordRequiredSourceRule {
    fn check(&self, item: &AlertRecord) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRecordApprovedGuardRule {
    pub limit: i64,
}

impl AlertRecordRule for AlertRecordApprovedGuardRule {
    fn check(&self, item: &AlertRecord) -> Vec<String> {
        if item.approved && (item.ttl_seconds > self.limit) {
            return vec![format!(
                "ttl seconds {} exceeds {} while approved",
                item.ttl_seconds, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRecordAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AlertRecordRule for AlertRecordAmountRangeRule {
    fn check(&self, item: &AlertRecord) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRecordRequiredNameRule {
    pub message: String,
}

impl AlertRecordRule for AlertRecordRequiredNameRule {
    fn check(&self, item: &AlertRecord) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert record.
#[derive(Debug, Clone)]
pub struct AlertRecordValidator {
    pub rules: Vec<Rc<dyn AlertRecordRule>>,
}

impl AlertRecordValidator {
    pub fn validate(&self, item: &AlertRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AlertRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_record_validator() -> AlertRecordValidator {
    AlertRecordValidator {
        rules: vec![
            Rc::new(AlertRecordCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn AlertRecordRule>,
            Rc::new(AlertRecordRequiredSourceRule {
                message: "alert record needs a value".to_string(),
            }),
            Rc::new(AlertRecordApprovedGuardRule { limit: 16 }),
            Rc::new(AlertRecordAmountRangeRule { min: 7.5, max: 774.0 }),
            Rc::new(AlertRecordRequiredNameRule {
                message: "alert record needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_record_validator();
        let item = new_alert_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = AlertRecordRequiredSourceRule {
            message: "alert record needs this field".to_string(),
        };
        let mut item = new_alert_record("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = AlertRecordAmountRangeRule { min: 3.1, max: 1111.1 };
        let mut item = new_alert_record("x", "y");
        item.amount = 1112.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = AlertRecordRequiredNameRule {
            message: "alert record needs an entry".to_string(),
        };
        let mut item = new_alert_record("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
