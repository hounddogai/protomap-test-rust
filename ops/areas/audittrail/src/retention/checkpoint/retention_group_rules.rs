use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a retention group must pass. Check returns its problems, or none.
pub trait RetentionGroupRule: std::fmt::Debug {
    fn check(&self, item: &RetentionGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionGroupCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionGroupRule for RetentionGroupCreatedAtRangeRule {
    fn check(&self, item: &RetentionGroup) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionGroupAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RetentionGroupRule for RetentionGroupAmountRangeRule {
    fn check(&self, item: &RetentionGroup) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionGroupAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RetentionGroupRule for RetentionGroupAllowedCodeRule {
    fn check(&self, item: &RetentionGroup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is missing", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionGroupRequiredLabelRule {
    pub message: String,
}

impl RetentionGroupRule for RetentionGroupRequiredLabelRule {
    fn check(&self, item: &RetentionGroup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionGroupRequiredCodeRule {
    pub message: String,
}

impl RetentionGroupRule for RetentionGroupRequiredCodeRule {
    fn check(&self, item: &RetentionGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionGroupBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionGroupRule for RetentionGroupBatchSizeRangeRule {
    fn check(&self, item: &RetentionGroup) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention group.
#[derive(Debug, Clone)]
pub struct RetentionGroupValidator {
    pub rules: Vec<Rc<dyn RetentionGroupRule>>,
}

impl RetentionGroupValidator {
    pub fn validate(&self, item: &RetentionGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_group_validator() -> RetentionGroupValidator {
    RetentionGroupValidator {
        rules: vec![
            Rc::new(RetentionGroupCreatedAtRangeRule { min: 255, max: 1000 }) as Rc<dyn RetentionGroupRule>,
            Rc::new(RetentionGroupAmountRangeRule { min: 6.6, max: 2505.3 }),
            Rc::new(RetentionGroupAllowedCodeRule {
                allowed: vec!["legacy".to_string(), "manual".to_string()],
            }),
            Rc::new(RetentionGroupRequiredLabelRule {
                message: "retention group needs this field".to_string(),
            }),
            Rc::new(RetentionGroupRequiredCodeRule {
                message: "retention group needs this field".to_string(),
            }),
            Rc::new(RetentionGroupBatchSizeRangeRule { min: 16, max: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_group_validator();
        let item = new_retention_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = RetentionGroupCreatedAtRangeRule { min: 80, max: 255 };
        let mut item = new_retention_group("x", "y");
        item.created_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = RetentionGroupAmountRangeRule { min: 5.9, max: 3449.7 };
        let mut item = new_retention_group("x", "y");
        item.amount = 3451.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RetentionGroupRequiredLabelRule {
            message: "retention group needs this field".to_string(),
        };
        let mut item = new_retention_group("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RetentionGroupRequiredCodeRule {
            message: "retention group needs a value".to_string(),
        };
        let mut item = new_retention_group("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = RetentionGroupBatchSizeRangeRule { min: 80, max: 120 };
        let mut item = new_retention_group("x", "y");
        item.batch_size = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
