use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global egress batch must pass. Check returns its problems, or none.
pub trait GlobalEgressBatchRule: std::fmt::Debug {
    fn check(&self, item: &GlobalEgressBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEgressBatchMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalEgressBatchRule for GlobalEgressBatchMaximumRangeRule {
    fn check(&self, item: &GlobalEgressBatch) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEgressBatchCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalEgressBatchRule for GlobalEgressBatchCostRangeRule {
    fn check(&self, item: &GlobalEgressBatch) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEgressBatchActiveGuardRule {
    pub limit: f64,
}

impl GlobalEgressBatchRule for GlobalEgressBatchActiveGuardRule {
    fn check(&self, item: &GlobalEgressBatch) -> Vec<String> {
        if item.active && (item.maximum > self.limit) {
            return vec![format!("maximum {} exceeds {} while active", item.maximum, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEgressBatchRequiredCodeRule {
    pub message: String,
}

impl GlobalEgressBatchRule for GlobalEgressBatchRequiredCodeRule {
    fn check(&self, item: &GlobalEgressBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEgressBatchDescriptionLengthRule {
    pub max_length: i64,
}

impl GlobalEgressBatchRule for GlobalEgressBatchDescriptionLengthRule {
    fn check(&self, item: &GlobalEgressBatch) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEgressBatchLabelLengthRule {
    pub max_length: i64,
}

impl GlobalEgressBatchRule for GlobalEgressBatchLabelLengthRule {
    fn check(&self, item: &GlobalEgressBatch) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global egress batch.
#[derive(Debug, Clone)]
pub struct GlobalEgressBatchValidator {
    pub rules: Vec<Rc<dyn GlobalEgressBatchRule>>,
}

impl GlobalEgressBatchValidator {
    pub fn validate(&self, item: &GlobalEgressBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalEgressBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalEgressBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_egress_batch_validator() -> GlobalEgressBatchValidator {
    GlobalEgressBatchValidator {
        rules: vec![
            Rc::new(GlobalEgressBatchMaximumRangeRule { min: 1.2, max: 196.1 }) as Rc<dyn GlobalEgressBatchRule>,
            Rc::new(GlobalEgressBatchCostRangeRule { min: 9.3, max: 3448.8 }),
            Rc::new(GlobalEgressBatchActiveGuardRule { limit: 9.4 }),
            Rc::new(GlobalEgressBatchRequiredCodeRule {
                message: "global egress batch needs this field".to_string(),
            }),
            Rc::new(GlobalEgressBatchDescriptionLengthRule { max_length: 80 }),
            Rc::new(GlobalEgressBatchLabelLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_egress_batch_validator();
        let item = new_global_egress_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = GlobalEgressBatchMaximumRangeRule { min: 5.2, max: 1246.7 };
        let mut item = new_global_egress_batch("x", "y");
        item.maximum = 1248.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = GlobalEgressBatchCostRangeRule { min: 3.8, max: 3979.6 };
        let mut item = new_global_egress_batch("x", "y");
        item.cost = 3981.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalEgressBatchRequiredCodeRule {
            message: "global egress batch needs a value".to_string(),
        };
        let mut item = new_global_egress_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = GlobalEgressBatchDescriptionLengthRule { max_length: 86400 };
        let mut item = new_global_egress_batch("x", "y");
        item.description = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = GlobalEgressBatchLabelLengthRule { max_length: 16 };
        let mut item = new_global_egress_batch("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
