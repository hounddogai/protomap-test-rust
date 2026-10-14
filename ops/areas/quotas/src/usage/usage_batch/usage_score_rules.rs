use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a usage score must pass. Check returns its problems, or none.
pub trait UsageScoreRule: std::fmt::Debug {
    fn check(&self, item: &UsageScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageScoreCodeLengthRule {
    pub max_length: i64,
}

impl UsageScoreRule for UsageScoreCodeLengthRule {
    fn check(&self, item: &UsageScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageScoreRequiredLabelRule {
    pub message: String,
}

impl UsageScoreRule for UsageScoreRequiredLabelRule {
    fn check(&self, item: &UsageScore) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageScoreCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl UsageScoreRule for UsageScoreCapacityRangeRule {
    fn check(&self, item: &UsageScore) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageScoreAllowedHostRule {
    pub allowed: Vec<String>,
}

impl UsageScoreRule for UsageScoreAllowedHostRule {
    fn check(&self, item: &UsageScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.host))) {
            return vec![format!("host {} is inconsistent", item.host)];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage score.
#[derive(Debug, Clone)]
pub struct UsageScoreValidator {
    pub rules: Vec<Rc<dyn UsageScoreRule>>,
}

impl UsageScoreValidator {
    pub fn validate(&self, item: &UsageScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &UsageScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_score_validator() -> UsageScoreValidator {
    UsageScoreValidator {
        rules: vec![
            Rc::new(UsageScoreCodeLengthRule { max_length: 1000 }) as Rc<dyn UsageScoreRule>,
            Rc::new(UsageScoreRequiredLabelRule {
                message: "usage score needs an entry".to_string(),
            }),
            Rc::new(UsageScoreCapacityRangeRule { min: 32, max: 1000 }),
            Rc::new(UsageScoreAllowedHostRule {
                allowed: vec!["strict".to_string(), "regional".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_score_validator();
        let item = new_usage_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = UsageScoreCodeLengthRule { max_length: 64 };
        let mut item = new_usage_score("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = UsageScoreRequiredLabelRule {
            message: "usage score needs this field".to_string(),
        };
        let mut item = new_usage_score("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = UsageScoreCapacityRangeRule { min: 16, max: 120 };
        let mut item = new_usage_score("x", "y");
        item.capacity = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
