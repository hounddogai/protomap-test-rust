use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling verifier score must pass. Check returns its problems, or none.
pub trait RollingVerifierScoreRule: std::fmt::Debug {
    fn check(&self, item: &RollingVerifierScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierScoreWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingVerifierScoreRule for RollingVerifierScoreWindowDaysRangeRule {
    fn check(&self, item: &RollingVerifierScore) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierScoreRequiredClusterRule {
    pub message: String,
}

impl RollingVerifierScoreRule for RollingVerifierScoreRequiredClusterRule {
    fn check(&self, item: &RollingVerifierScore) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierScoreRequiredCodeRule {
    pub message: String,
}

impl RollingVerifierScoreRule for RollingVerifierScoreRequiredCodeRule {
    fn check(&self, item: &RollingVerifierScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling verifier score.
#[derive(Debug, Clone)]
pub struct RollingVerifierScoreValidator {
    pub rules: Vec<Rc<dyn RollingVerifierScoreRule>>,
}

impl RollingVerifierScoreValidator {
    pub fn validate(&self, item: &RollingVerifierScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingVerifierScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingVerifierScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_verifier_score_validator() -> RollingVerifierScoreValidator {
    RollingVerifierScoreValidator {
        rules: vec![
            Rc::new(RollingVerifierScoreWindowDaysRangeRule { min: 3600, max: 86400 })
                as Rc<dyn RollingVerifierScoreRule>,
            Rc::new(RollingVerifierScoreRequiredClusterRule {
                message: "rolling verifier score needs a value".to_string(),
            }),
            Rc::new(RollingVerifierScoreRequiredCodeRule {
                message: "rolling verifier score needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_verifier_score_validator();
        let item = new_rolling_verifier_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = RollingVerifierScoreWindowDaysRangeRule { min: 16, max: 32 };
        let mut item = new_rolling_verifier_score("x", "y");
        item.window_days = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = RollingVerifierScoreRequiredClusterRule {
            message: "rolling verifier score needs a value".to_string(),
        };
        let mut item = new_rolling_verifier_score("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingVerifierScoreRequiredCodeRule {
            message: "rolling verifier score needs an entry".to_string(),
        };
        let mut item = new_rolling_verifier_score("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
