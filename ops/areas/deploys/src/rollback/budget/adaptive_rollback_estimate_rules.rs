use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive rollback estimate must pass. Check returns its problems, or none.
pub trait AdaptiveRollbackEstimateRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveRollbackEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRollbackEstimateMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveRollbackEstimateRule for AdaptiveRollbackEstimateMarginRangeRule {
    fn check(&self, item: &AdaptiveRollbackEstimate) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRollbackEstimateRequiredLabelRule {
    pub message: String,
}

impl AdaptiveRollbackEstimateRule for AdaptiveRollbackEstimateRequiredLabelRule {
    fn check(&self, item: &AdaptiveRollbackEstimate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRollbackEstimateNameLengthRule {
    pub max_length: i64,
}

impl AdaptiveRollbackEstimateRule for AdaptiveRollbackEstimateNameLengthRule {
    fn check(&self, item: &AdaptiveRollbackEstimate) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRollbackEstimateLocaleLengthRule {
    pub max_length: i64,
}

impl AdaptiveRollbackEstimateRule for AdaptiveRollbackEstimateLocaleLengthRule {
    fn check(&self, item: &AdaptiveRollbackEstimate) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive rollback estimate.
#[derive(Debug, Clone)]
pub struct AdaptiveRollbackEstimateValidator {
    pub rules: Vec<Rc<dyn AdaptiveRollbackEstimateRule>>,
}

impl AdaptiveRollbackEstimateValidator {
    pub fn validate(&self, item: &AdaptiveRollbackEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveRollbackEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveRollbackEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_rollback_estimate_validator() -> AdaptiveRollbackEstimateValidator {
    AdaptiveRollbackEstimateValidator {
        rules: vec![
            Rc::new(AdaptiveRollbackEstimateMarginRangeRule { min: 0.3, max: 4565.6 })
                as Rc<dyn AdaptiveRollbackEstimateRule>,
            Rc::new(AdaptiveRollbackEstimateRequiredLabelRule {
                message: "adaptive rollback estimate needs an entry".to_string(),
            }),
            Rc::new(AdaptiveRollbackEstimateNameLengthRule { max_length: 255 }),
            Rc::new(AdaptiveRollbackEstimateLocaleLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_rollback_estimate_validator();
        let item = new_adaptive_rollback_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = AdaptiveRollbackEstimateMarginRangeRule { min: 5.7, max: 4429.6 };
        let mut item = new_adaptive_rollback_estimate("x", "y");
        item.margin = 4431.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = AdaptiveRollbackEstimateRequiredLabelRule {
            message: "adaptive rollback estimate needs a value".to_string(),
        };
        let mut item = new_adaptive_rollback_estimate("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = AdaptiveRollbackEstimateNameLengthRule { max_length: 1000 };
        let mut item = new_adaptive_rollback_estimate("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = AdaptiveRollbackEstimateLocaleLengthRule { max_length: 16 };
        let mut item = new_adaptive_rollback_estimate("x", "y");
        item.locale = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
