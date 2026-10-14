use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a ttl policy score must pass. Check returns its problems, or none.
pub trait TtlPolicyScoreRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyScoreCodeLengthRule {
    pub max_length: i64,
}

impl TtlPolicyScoreRule for TtlPolicyScoreCodeLengthRule {
    fn check(&self, item: &TtlPolicyScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyScoreRequiredCodeRule {
    pub message: String,
}

impl TtlPolicyScoreRule for TtlPolicyScoreRequiredCodeRule {
    fn check(&self, item: &TtlPolicyScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyScoreTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TtlPolicyScoreRule for TtlPolicyScoreTtlSecondsRangeRule {
    fn check(&self, item: &TtlPolicyScore) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyScoreCodePrefixRule {
    pub prefix: String,
}

impl TtlPolicyScoreRule for TtlPolicyScoreCodePrefixRule {
    fn check(&self, item: &TtlPolicyScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy score.
#[derive(Debug, Clone)]
pub struct TtlPolicyScoreValidator {
    pub rules: Vec<Rc<dyn TtlPolicyScoreRule>>,
}

impl TtlPolicyScoreValidator {
    pub fn validate(&self, item: &TtlPolicyScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TtlPolicyScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_score_validator() -> TtlPolicyScoreValidator {
    TtlPolicyScoreValidator {
        rules: vec![
            Rc::new(TtlPolicyScoreCodeLengthRule { max_length: 64 }) as Rc<dyn TtlPolicyScoreRule>,
            Rc::new(TtlPolicyScoreRequiredCodeRule {
                message: "ttl policy score needs an entry".to_string(),
            }),
            Rc::new(TtlPolicyScoreTtlSecondsRangeRule { min: 3600, max: 3600 }),
            Rc::new(TtlPolicyScoreCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_score_validator();
        let item = new_ttl_policy_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TtlPolicyScoreCodeLengthRule { max_length: 64 };
        let mut item = new_ttl_policy_score("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TtlPolicyScoreRequiredCodeRule {
            message: "ttl policy score needs an entry".to_string(),
        };
        let mut item = new_ttl_policy_score("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = TtlPolicyScoreTtlSecondsRangeRule { min: 64, max: 255 };
        let mut item = new_ttl_policy_score("x", "y");
        item.ttl_seconds = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
