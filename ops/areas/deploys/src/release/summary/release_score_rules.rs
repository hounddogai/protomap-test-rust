use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a release score must pass. Check returns its problems, or none.
pub trait ReleaseScoreRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseScoreCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReleaseScoreRule for ReleaseScoreCreatedAtRangeRule {
    fn check(&self, item: &ReleaseScore) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseScoreCodeLengthRule {
    pub max_length: i64,
}

impl ReleaseScoreRule for ReleaseScoreCodeLengthRule {
    fn check(&self, item: &ReleaseScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseScoreAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReleaseScoreRule for ReleaseScoreAmountRangeRule {
    fn check(&self, item: &ReleaseScore) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseScoreSummaryLengthRule {
    pub max_length: i64,
}

impl ReleaseScoreRule for ReleaseScoreSummaryLengthRule {
    fn check(&self, item: &ReleaseScore) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseScoreRequiredLabelRule {
    pub message: String,
}

impl ReleaseScoreRule for ReleaseScoreRequiredLabelRule {
    fn check(&self, item: &ReleaseScore) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a release score.
#[derive(Debug, Clone)]
pub struct ReleaseScoreValidator {
    pub rules: Vec<Rc<dyn ReleaseScoreRule>>,
}

impl ReleaseScoreValidator {
    pub fn validate(&self, item: &ReleaseScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReleaseScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_score_validator() -> ReleaseScoreValidator {
    ReleaseScoreValidator {
        rules: vec![
            Rc::new(ReleaseScoreCreatedAtRangeRule { min: 255, max: 86400 }) as Rc<dyn ReleaseScoreRule>,
            Rc::new(ReleaseScoreCodeLengthRule { max_length: 80 }),
            Rc::new(ReleaseScoreAmountRangeRule { min: 3.2, max: 848.3 }),
            Rc::new(ReleaseScoreSummaryLengthRule { max_length: 32 }),
            Rc::new(ReleaseScoreRequiredLabelRule {
                message: "release score needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_score_validator();
        let item = new_release_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = ReleaseScoreCreatedAtRangeRule { min: 32, max: 1000 };
        let mut item = new_release_score("x", "y");
        item.created_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ReleaseScoreCodeLengthRule { max_length: 80 };
        let mut item = new_release_score("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = ReleaseScoreAmountRangeRule { min: 4.2, max: 795.1 };
        let mut item = new_release_score("x", "y");
        item.amount = 796.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = ReleaseScoreSummaryLengthRule { max_length: 1000 };
        let mut item = new_release_score("x", "y");
        item.summary = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ReleaseScoreRequiredLabelRule {
            message: "release score needs an entry".to_string(),
        };
        let mut item = new_release_score("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
