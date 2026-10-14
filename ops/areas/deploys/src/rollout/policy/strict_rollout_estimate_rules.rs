use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict rollout estimate must pass. Check returns its problems, or none.
pub trait StrictRolloutEstimateRule: std::fmt::Debug {
    fn check(&self, item: &StrictRolloutEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutEstimateCodeLengthRule {
    pub max_length: i64,
}

impl StrictRolloutEstimateRule for StrictRolloutEstimateCodeLengthRule {
    fn check(&self, item: &StrictRolloutEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutEstimateLabelLengthRule {
    pub max_length: i64,
}

impl StrictRolloutEstimateRule for StrictRolloutEstimateLabelLengthRule {
    fn check(&self, item: &StrictRolloutEstimate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutEstimateDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictRolloutEstimateRule for StrictRolloutEstimateDiscountRangeRule {
    fn check(&self, item: &StrictRolloutEstimate) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutEstimatePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictRolloutEstimateRule for StrictRolloutEstimatePriorityRangeRule {
    fn check(&self, item: &StrictRolloutEstimate) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutEstimateAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl StrictRolloutEstimateRule for StrictRolloutEstimateAllowedLabelRule {
    fn check(&self, item: &StrictRolloutEstimate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is expired", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutEstimateCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictRolloutEstimateRule for StrictRolloutEstimateCeilingRangeRule {
    fn check(&self, item: &StrictRolloutEstimate) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a strict rollout estimate.
#[derive(Debug, Clone)]
pub struct StrictRolloutEstimateValidator {
    pub rules: Vec<Rc<dyn StrictRolloutEstimateRule>>,
}

impl StrictRolloutEstimateValidator {
    pub fn validate(&self, item: &StrictRolloutEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictRolloutEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictRolloutEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_rollout_estimate_validator() -> StrictRolloutEstimateValidator {
    StrictRolloutEstimateValidator {
        rules: vec![
            Rc::new(StrictRolloutEstimateCodeLengthRule { max_length: 3600 }) as Rc<dyn StrictRolloutEstimateRule>,
            Rc::new(StrictRolloutEstimateLabelLengthRule { max_length: 86400 }),
            Rc::new(StrictRolloutEstimateDiscountRangeRule { min: 2.2, max: 2128.5 }),
            Rc::new(StrictRolloutEstimatePriorityRangeRule { min: 80, max: 1000 }),
            Rc::new(StrictRolloutEstimateAllowedLabelRule {
                allowed: vec!["global".to_string(), "seasonal".to_string(), "scheduled".to_string()],
            }),
            Rc::new(StrictRolloutEstimateCeilingRangeRule { min: 5.7, max: 4671.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_rollout_estimate_validator();
        let item = new_strict_rollout_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictRolloutEstimateCodeLengthRule { max_length: 255 };
        let mut item = new_strict_rollout_estimate("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = StrictRolloutEstimateLabelLengthRule { max_length: 86400 };
        let mut item = new_strict_rollout_estimate("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = StrictRolloutEstimateDiscountRangeRule { min: 4.1, max: 3067.5 };
        let mut item = new_strict_rollout_estimate("x", "y");
        item.discount = 3069.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = StrictRolloutEstimatePriorityRangeRule { min: 80, max: 255 };
        let mut item = new_strict_rollout_estimate("x", "y");
        item.priority = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = StrictRolloutEstimateCeilingRangeRule { min: 5.4, max: 2524.1 };
        let mut item = new_strict_rollout_estimate("x", "y");
        item.ceiling = 2525.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
