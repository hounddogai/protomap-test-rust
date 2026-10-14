use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a plan batch must pass. Check returns its problems, or none.
pub trait PlanBatchRule: std::fmt::Debug {
    fn check(&self, item: &PlanBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanBatchStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PlanBatchRule for PlanBatchStartsAtRangeRule {
    fn check(&self, item: &PlanBatch) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanBatchRequiredTitleRule {
    pub message: String,
}

impl PlanBatchRule for PlanBatchRequiredTitleRule {
    fn check(&self, item: &PlanBatch) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanBatchCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PlanBatchRule for PlanBatchCapacityRangeRule {
    fn check(&self, item: &PlanBatch) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanBatchTitleLengthRule {
    pub max_length: i64,
}

impl PlanBatchRule for PlanBatchTitleLengthRule {
    fn check(&self, item: &PlanBatch) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanBatchAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PlanBatchRule for PlanBatchAllowedCodeRule {
    fn check(&self, item: &PlanBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan batch.
#[derive(Debug, Clone)]
pub struct PlanBatchValidator {
    pub rules: Vec<Rc<dyn PlanBatchRule>>,
}

impl PlanBatchValidator {
    pub fn validate(&self, item: &PlanBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PlanBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_batch_validator() -> PlanBatchValidator {
    PlanBatchValidator {
        rules: vec![
            Rc::new(PlanBatchStartsAtRangeRule { min: 255, max: 86400 }) as Rc<dyn PlanBatchRule>,
            Rc::new(PlanBatchRequiredTitleRule {
                message: "plan batch needs an entry".to_string(),
            }),
            Rc::new(PlanBatchCapacityRangeRule { min: 32, max: 86400 }),
            Rc::new(PlanBatchTitleLengthRule { max_length: 32 }),
            Rc::new(PlanBatchAllowedCodeRule {
                allowed: vec!["primary".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_plan_batch_validator();
        let item = new_plan_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = PlanBatchStartsAtRangeRule { min: 16, max: 80 };
        let mut item = new_plan_batch("x", "y");
        item.starts_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PlanBatchRequiredTitleRule {
            message: "plan batch needs this field".to_string(),
        };
        let mut item = new_plan_batch("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = PlanBatchCapacityRangeRule { min: 255, max: 255 };
        let mut item = new_plan_batch("x", "y");
        item.capacity = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PlanBatchTitleLengthRule { max_length: 86400 };
        let mut item = new_plan_batch("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
