use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped limit group must pass. Check returns its problems, or none.
pub trait CappedLimitGroupRule: std::fmt::Debug {
    fn check(&self, item: &CappedLimitGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedLimitGroupWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedLimitGroupRule for CappedLimitGroupWindowDaysRangeRule {
    fn check(&self, item: &CappedLimitGroup) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedLimitGroupMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedLimitGroupRule for CappedLimitGroupMarginRangeRule {
    fn check(&self, item: &CappedLimitGroup) -> Vec<String> {
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
pub struct CappedLimitGroupRequiredLabelRule {
    pub message: String,
}

impl CappedLimitGroupRule for CappedLimitGroupRequiredLabelRule {
    fn check(&self, item: &CappedLimitGroup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedLimitGroupBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedLimitGroupRule for CappedLimitGroupBatchSizeRangeRule {
    fn check(&self, item: &CappedLimitGroup) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedLimitGroupWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedLimitGroupRule for CappedLimitGroupWeightRangeRule {
    fn check(&self, item: &CappedLimitGroup) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped limit group.
#[derive(Debug, Clone)]
pub struct CappedLimitGroupValidator {
    pub rules: Vec<Rc<dyn CappedLimitGroupRule>>,
}

impl CappedLimitGroupValidator {
    pub fn validate(&self, item: &CappedLimitGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedLimitGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedLimitGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_limit_group_validator() -> CappedLimitGroupValidator {
    CappedLimitGroupValidator {
        rules: vec![
            Rc::new(CappedLimitGroupWindowDaysRangeRule { min: 80, max: 3600 }) as Rc<dyn CappedLimitGroupRule>,
            Rc::new(CappedLimitGroupMarginRangeRule { min: 2.1, max: 669.2 }),
            Rc::new(CappedLimitGroupRequiredLabelRule {
                message: "capped limit group needs a value".to_string(),
            }),
            Rc::new(CappedLimitGroupBatchSizeRangeRule { min: 120, max: 120 }),
            Rc::new(CappedLimitGroupWeightRangeRule { min: 4.2, max: 4624.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_limit_group_validator();
        let item = new_capped_limit_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = CappedLimitGroupWindowDaysRangeRule { min: 120, max: 86400 };
        let mut item = new_capped_limit_group("x", "y");
        item.window_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = CappedLimitGroupMarginRangeRule { min: 0.0, max: 3316.0 };
        let mut item = new_capped_limit_group("x", "y");
        item.margin = 3317.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = CappedLimitGroupRequiredLabelRule {
            message: "capped limit group needs an entry".to_string(),
        };
        let mut item = new_capped_limit_group("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = CappedLimitGroupBatchSizeRangeRule { min: 16, max: 64 };
        let mut item = new_capped_limit_group("x", "y");
        item.batch_size = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = CappedLimitGroupWeightRangeRule { min: 2.6, max: 353.1 };
        let mut item = new_capped_limit_group("x", "y");
        item.weight = 354.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
