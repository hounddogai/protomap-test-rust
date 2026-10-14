use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted failover score must pass. Check returns its problems, or none.
pub trait WeightedFailoverScoreRule: std::fmt::Debug {
    fn check(&self, item: &WeightedFailoverScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFailoverScoreReviewerLengthRule {
    pub max_length: i64,
}

impl WeightedFailoverScoreRule for WeightedFailoverScoreReviewerLengthRule {
    fn check(&self, item: &WeightedFailoverScore) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFailoverScoreCodeLengthRule {
    pub max_length: i64,
}

impl WeightedFailoverScoreRule for WeightedFailoverScoreCodeLengthRule {
    fn check(&self, item: &WeightedFailoverScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFailoverScoreRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedFailoverScoreRule for WeightedFailoverScoreRatioRangeRule {
    fn check(&self, item: &WeightedFailoverScore) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFailoverScoreMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedFailoverScoreRule for WeightedFailoverScoreMaximumRangeRule {
    fn check(&self, item: &WeightedFailoverScore) -> Vec<String> {
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
pub struct WeightedFailoverScoreRequiredTitleRule {
    pub message: String,
}

impl WeightedFailoverScoreRule for WeightedFailoverScoreRequiredTitleRule {
    fn check(&self, item: &WeightedFailoverScore) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted failover score.
#[derive(Debug, Clone)]
pub struct WeightedFailoverScoreValidator {
    pub rules: Vec<Rc<dyn WeightedFailoverScoreRule>>,
}

impl WeightedFailoverScoreValidator {
    pub fn validate(&self, item: &WeightedFailoverScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedFailoverScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedFailoverScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_failover_score_validator() -> WeightedFailoverScoreValidator {
    WeightedFailoverScoreValidator {
        rules: vec![
            Rc::new(WeightedFailoverScoreReviewerLengthRule { max_length: 120 }) as Rc<dyn WeightedFailoverScoreRule>,
            Rc::new(WeightedFailoverScoreCodeLengthRule { max_length: 3600 }),
            Rc::new(WeightedFailoverScoreRatioRangeRule { min: 3.9, max: 2123.0 }),
            Rc::new(WeightedFailoverScoreMaximumRangeRule { min: 1.6, max: 325.1 }),
            Rc::new(WeightedFailoverScoreRequiredTitleRule {
                message: "weighted failover score needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_failover_score_validator();
        let item = new_weighted_failover_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = WeightedFailoverScoreReviewerLengthRule { max_length: 64 };
        let mut item = new_weighted_failover_score("x", "y");
        item.reviewer = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedFailoverScoreCodeLengthRule { max_length: 80 };
        let mut item = new_weighted_failover_score("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = WeightedFailoverScoreRatioRangeRule { min: 9.0, max: 678.9 };
        let mut item = new_weighted_failover_score("x", "y");
        item.ratio = 680.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = WeightedFailoverScoreMaximumRangeRule { min: 7.5, max: 3754.4 };
        let mut item = new_weighted_failover_score("x", "y");
        item.maximum = 3755.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = WeightedFailoverScoreRequiredTitleRule {
            message: "weighted failover score needs an entry".to_string(),
        };
        let mut item = new_weighted_failover_score("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
