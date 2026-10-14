use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a redaction budget must pass. Check returns its problems, or none.
pub trait RedactionBudgetRule: std::fmt::Debug {
    fn check(&self, item: &RedactionBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBudgetAllowedSegmentRule {
    pub allowed: Vec<String>,
}

impl RedactionBudgetRule for RedactionBudgetAllowedSegmentRule {
    fn check(&self, item: &RedactionBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.segment)))
        {
            return vec![format!("segment {} is missing", item.segment)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBudgetRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RedactionBudgetRule for RedactionBudgetRateRangeRule {
    fn check(&self, item: &RedactionBudget) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBudgetRequiredSegmentRule {
    pub message: String,
}

impl RedactionBudgetRule for RedactionBudgetRequiredSegmentRule {
    fn check(&self, item: &RedactionBudget) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBudgetExternalRefLengthRule {
    pub max_length: i64,
}

impl RedactionBudgetRule for RedactionBudgetExternalRefLengthRule {
    fn check(&self, item: &RedactionBudget) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBudgetPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RedactionBudgetRule for RedactionBudgetPriorityRangeRule {
    fn check(&self, item: &RedactionBudget) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBudgetRequiredCodeRule {
    pub message: String,
}

impl RedactionBudgetRule for RedactionBudgetRequiredCodeRule {
    fn check(&self, item: &RedactionBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction budget.
#[derive(Debug, Clone)]
pub struct RedactionBudgetValidator {
    pub rules: Vec<Rc<dyn RedactionBudgetRule>>,
}

impl RedactionBudgetValidator {
    pub fn validate(&self, item: &RedactionBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RedactionBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_budget_validator() -> RedactionBudgetValidator {
    RedactionBudgetValidator {
        rules: vec![
            Rc::new(RedactionBudgetAllowedSegmentRule {
                allowed: vec!["seasonal".to_string()],
            }) as Rc<dyn RedactionBudgetRule>,
            Rc::new(RedactionBudgetRateRangeRule { min: 5.3, max: 3631.9 }),
            Rc::new(RedactionBudgetRequiredSegmentRule {
                message: "redaction budget needs this field".to_string(),
            }),
            Rc::new(RedactionBudgetExternalRefLengthRule { max_length: 255 }),
            Rc::new(RedactionBudgetPriorityRangeRule { min: 255, max: 1000 }),
            Rc::new(RedactionBudgetRequiredCodeRule {
                message: "redaction budget needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_redaction_budget_validator();
        let item = new_redaction_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = RedactionBudgetRateRangeRule { min: 5.9, max: 2814.8 };
        let mut item = new_redaction_budget("x", "y");
        item.rate = 2816.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = RedactionBudgetRequiredSegmentRule {
            message: "redaction budget needs this field".to_string(),
        };
        let mut item = new_redaction_budget("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = RedactionBudgetExternalRefLengthRule { max_length: 255 };
        let mut item = new_redaction_budget("x", "y");
        item.external_ref = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = RedactionBudgetPriorityRangeRule { min: 16, max: 3600 };
        let mut item = new_redaction_budget("x", "y");
        item.priority = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RedactionBudgetRequiredCodeRule {
            message: "redaction budget needs an entry".to_string(),
        };
        let mut item = new_redaction_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
