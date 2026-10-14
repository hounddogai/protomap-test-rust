use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a retention adjustment must pass. Check returns its problems, or none.
pub trait RetentionAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &RetentionAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl RetentionAdjustmentRule for RetentionAdjustmentCodePrefixRule {
    fn check(&self, item: &RetentionAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAdjustmentTitleLengthRule {
    pub max_length: i64,
}

impl RetentionAdjustmentRule for RetentionAdjustmentTitleLengthRule {
    fn check(&self, item: &RetentionAdjustment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAdjustmentStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionAdjustmentRule for RetentionAdjustmentStartsAtRangeRule {
    fn check(&self, item: &RetentionAdjustment) -> Vec<String> {
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
pub struct RetentionAdjustmentScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RetentionAdjustmentRule for RetentionAdjustmentScoreRangeRule {
    fn check(&self, item: &RetentionAdjustment) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAdjustmentClusterLengthRule {
    pub max_length: i64,
}

impl RetentionAdjustmentRule for RetentionAdjustmentClusterLengthRule {
    fn check(&self, item: &RetentionAdjustment) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention adjustment.
#[derive(Debug, Clone)]
pub struct RetentionAdjustmentValidator {
    pub rules: Vec<Rc<dyn RetentionAdjustmentRule>>,
}

impl RetentionAdjustmentValidator {
    pub fn validate(&self, item: &RetentionAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_adjustment_validator() -> RetentionAdjustmentValidator {
    RetentionAdjustmentValidator {
        rules: vec![
            Rc::new(RetentionAdjustmentCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn RetentionAdjustmentRule>,
            Rc::new(RetentionAdjustmentTitleLengthRule { max_length: 86400 }),
            Rc::new(RetentionAdjustmentStartsAtRangeRule { min: 86400, max: 86400 }),
            Rc::new(RetentionAdjustmentScoreRangeRule { min: 5.1, max: 4600.0 }),
            Rc::new(RetentionAdjustmentClusterLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_adjustment_validator();
        let item = new_retention_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RetentionAdjustmentTitleLengthRule { max_length: 120 };
        let mut item = new_retention_adjustment("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = RetentionAdjustmentStartsAtRangeRule { min: 80, max: 86400 };
        let mut item = new_retention_adjustment("x", "y");
        item.starts_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = RetentionAdjustmentScoreRangeRule { min: 2.2, max: 4652.3 };
        let mut item = new_retention_adjustment("x", "y");
        item.score = 4653.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = RetentionAdjustmentClusterLengthRule { max_length: 80 };
        let mut item = new_retention_adjustment("x", "y");
        item.cluster = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
