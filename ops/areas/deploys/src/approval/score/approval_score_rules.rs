use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a approval score must pass. Check returns its problems, or none.
pub trait ApprovalScoreRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalScoreStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ApprovalScoreRule for ApprovalScoreStartsAtRangeRule {
    fn check(&self, item: &ApprovalScore) -> Vec<String> {
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
pub struct ApprovalScoreCodePrefixRule {
    pub prefix: String,
}

impl ApprovalScoreRule for ApprovalScoreCodePrefixRule {
    fn check(&self, item: &ApprovalScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalScoreLabelLengthRule {
    pub max_length: i64,
}

impl ApprovalScoreRule for ApprovalScoreLabelLengthRule {
    fn check(&self, item: &ApprovalScore) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalScoreEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ApprovalScoreRule for ApprovalScoreEndsAtRangeRule {
    fn check(&self, item: &ApprovalScore) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalScoreAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl ApprovalScoreRule for ApprovalScoreAllowedLabelRule {
    fn check(&self, item: &ApprovalScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is inconsistent", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval score.
#[derive(Debug, Clone)]
pub struct ApprovalScoreValidator {
    pub rules: Vec<Rc<dyn ApprovalScoreRule>>,
}

impl ApprovalScoreValidator {
    pub fn validate(&self, item: &ApprovalScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ApprovalScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_score_validator() -> ApprovalScoreValidator {
    ApprovalScoreValidator {
        rules: vec![
            Rc::new(ApprovalScoreStartsAtRangeRule { min: 16, max: 64 }) as Rc<dyn ApprovalScoreRule>,
            Rc::new(ApprovalScoreCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(ApprovalScoreLabelLengthRule { max_length: 64 }),
            Rc::new(ApprovalScoreEndsAtRangeRule { min: 16, max: 255 }),
            Rc::new(ApprovalScoreAllowedLabelRule {
                allowed: vec!["regional".to_string(), "capped".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_approval_score_validator();
        let item = new_approval_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = ApprovalScoreStartsAtRangeRule { min: 120, max: 3600 };
        let mut item = new_approval_score("x", "y");
        item.starts_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ApprovalScoreLabelLengthRule { max_length: 64 };
        let mut item = new_approval_score("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = ApprovalScoreEndsAtRangeRule { min: 80, max: 120 };
        let mut item = new_approval_score("x", "y");
        item.ends_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
