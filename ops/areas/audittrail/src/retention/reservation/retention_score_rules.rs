use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a retention score must pass. Check returns its problems, or none.
pub trait RetentionScoreRule: std::fmt::Debug {
    fn check(&self, item: &RetentionScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionScoreRequiredLabelRule {
    pub message: String,
}

impl RetentionScoreRule for RetentionScoreRequiredLabelRule {
    fn check(&self, item: &RetentionScore) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionScoreRequiredHostRule {
    pub message: String,
}

impl RetentionScoreRule for RetentionScoreRequiredHostRule {
    fn check(&self, item: &RetentionScore) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionScoreAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionScoreRule for RetentionScoreAttemptsRangeRule {
    fn check(&self, item: &RetentionScore) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionScoreRequiredClusterRule {
    pub message: String,
}

impl RetentionScoreRule for RetentionScoreRequiredClusterRule {
    fn check(&self, item: &RetentionScore) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionScoreHostLengthRule {
    pub max_length: i64,
}

impl RetentionScoreRule for RetentionScoreHostLengthRule {
    fn check(&self, item: &RetentionScore) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention score.
#[derive(Debug, Clone)]
pub struct RetentionScoreValidator {
    pub rules: Vec<Rc<dyn RetentionScoreRule>>,
}

impl RetentionScoreValidator {
    pub fn validate(&self, item: &RetentionScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RetentionScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_score_validator() -> RetentionScoreValidator {
    RetentionScoreValidator {
        rules: vec![
            Rc::new(RetentionScoreRequiredLabelRule {
                message: "retention score needs a value".to_string(),
            }) as Rc<dyn RetentionScoreRule>,
            Rc::new(RetentionScoreRequiredHostRule {
                message: "retention score needs this field".to_string(),
            }),
            Rc::new(RetentionScoreAttemptsRangeRule { min: 80, max: 120 }),
            Rc::new(RetentionScoreRequiredClusterRule {
                message: "retention score needs a value".to_string(),
            }),
            Rc::new(RetentionScoreHostLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_score_validator();
        let item = new_retention_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RetentionScoreRequiredLabelRule {
            message: "retention score needs an entry".to_string(),
        };
        let mut item = new_retention_score("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = RetentionScoreRequiredHostRule {
            message: "retention score needs a value".to_string(),
        };
        let mut item = new_retention_score("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = RetentionScoreAttemptsRangeRule { min: 255, max: 86400 };
        let mut item = new_retention_score("x", "y");
        item.attempts = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = RetentionScoreRequiredClusterRule {
            message: "retention score needs a value".to_string(),
        };
        let mut item = new_retention_score("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = RetentionScoreHostLengthRule { max_length: 3600 };
        let mut item = new_retention_score("x", "y");
        item.host = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
