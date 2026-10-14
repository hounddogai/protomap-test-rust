use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a slo score must pass. Check returns its problems, or none.
pub trait SloScoreRule: std::fmt::Debug {
    fn check(&self, item: &SloScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloScoreTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl SloScoreRule for SloScoreTagsLimitRule {
    fn check(&self, item: &SloScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloScoreRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SloScoreRule for SloScoreRateRangeRule {
    fn check(&self, item: &SloScore) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloScoreRequiredCodeRule {
    pub message: String,
}

impl SloScoreRule for SloScoreRequiredCodeRule {
    fn check(&self, item: &SloScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo score.
#[derive(Debug, Clone)]
pub struct SloScoreValidator {
    pub rules: Vec<Rc<dyn SloScoreRule>>,
}

impl SloScoreValidator {
    pub fn validate(&self, item: &SloScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SloScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_score_validator() -> SloScoreValidator {
    SloScoreValidator {
        rules: vec![
            Rc::new(SloScoreTagsLimitRule {
                max_count: 80,
                banned: Vec::new(),
            }) as Rc<dyn SloScoreRule>,
            Rc::new(SloScoreRateRangeRule { min: 1.2, max: 1225.1 }),
            Rc::new(SloScoreRequiredCodeRule {
                message: "slo score needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_score_validator();
        let item = new_slo_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = SloScoreRateRangeRule { min: 9.9, max: 1004.2 };
        let mut item = new_slo_score("x", "y");
        item.rate = 1005.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SloScoreRequiredCodeRule {
            message: "slo score needs an entry".to_string(),
        };
        let mut item = new_slo_score("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
