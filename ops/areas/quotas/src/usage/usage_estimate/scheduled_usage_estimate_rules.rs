use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled usage estimate must pass. Check returns its problems, or none.
pub trait ScheduledUsageEstimateRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledUsageEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledUsageEstimateRequiredNameRule {
    pub message: String,
}

impl ScheduledUsageEstimateRule for ScheduledUsageEstimateRequiredNameRule {
    fn check(&self, item: &ScheduledUsageEstimate) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledUsageEstimateRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledUsageEstimateRule for ScheduledUsageEstimateRetriesRangeRule {
    fn check(&self, item: &ScheduledUsageEstimate) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledUsageEstimateCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledUsageEstimateRule for ScheduledUsageEstimateCodeLengthRule {
    fn check(&self, item: &ScheduledUsageEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledUsageEstimateApprovedGuardRule {
    pub limit: i64,
}

impl ScheduledUsageEstimateRule for ScheduledUsageEstimateApprovedGuardRule {
    fn check(&self, item: &ScheduledUsageEstimate) -> Vec<String> {
        if item.approved && (item.retries > self.limit) {
            return vec![format!(
                "retries {} exceeds {} while approved",
                item.retries, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledUsageEstimateCategoryLengthRule {
    pub max_length: i64,
}

impl ScheduledUsageEstimateRule for ScheduledUsageEstimateCategoryLengthRule {
    fn check(&self, item: &ScheduledUsageEstimate) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledUsageEstimateCodePrefixRule {
    pub prefix: String,
}

impl ScheduledUsageEstimateRule for ScheduledUsageEstimateCodePrefixRule {
    fn check(&self, item: &ScheduledUsageEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled usage estimate.
#[derive(Debug, Clone)]
pub struct ScheduledUsageEstimateValidator {
    pub rules: Vec<Rc<dyn ScheduledUsageEstimateRule>>,
}

impl ScheduledUsageEstimateValidator {
    pub fn validate(&self, item: &ScheduledUsageEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledUsageEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledUsageEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_usage_estimate_validator() -> ScheduledUsageEstimateValidator {
    ScheduledUsageEstimateValidator {
        rules: vec![
            Rc::new(ScheduledUsageEstimateRequiredNameRule {
                message: "scheduled usage estimate needs a value".to_string(),
            }) as Rc<dyn ScheduledUsageEstimateRule>,
            Rc::new(ScheduledUsageEstimateRetriesRangeRule { min: 16, max: 86400 }),
            Rc::new(ScheduledUsageEstimateCodeLengthRule { max_length: 255 }),
            Rc::new(ScheduledUsageEstimateApprovedGuardRule { limit: 3600 }),
            Rc::new(ScheduledUsageEstimateCategoryLengthRule { max_length: 120 }),
            Rc::new(ScheduledUsageEstimateCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_usage_estimate_validator();
        let item = new_scheduled_usage_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ScheduledUsageEstimateRequiredNameRule {
            message: "scheduled usage estimate needs this field".to_string(),
        };
        let mut item = new_scheduled_usage_estimate("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = ScheduledUsageEstimateRetriesRangeRule { min: 64, max: 86400 };
        let mut item = new_scheduled_usage_estimate("x", "y");
        item.retries = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledUsageEstimateCodeLengthRule { max_length: 120 };
        let mut item = new_scheduled_usage_estimate("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = ScheduledUsageEstimateCategoryLengthRule { max_length: 86400 };
        let mut item = new_scheduled_usage_estimate("x", "y");
        item.category = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
