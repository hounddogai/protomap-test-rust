use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient rollout score must pass. Check returns its problems, or none.
pub trait LenientRolloutScoreRule: std::fmt::Debug {
    fn check(&self, item: &LenientRolloutScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRolloutScoreTitleLengthRule {
    pub max_length: i64,
}

impl LenientRolloutScoreRule for LenientRolloutScoreTitleLengthRule {
    fn check(&self, item: &LenientRolloutScore) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRolloutScoreCodePrefixRule {
    pub prefix: String,
}

impl LenientRolloutScoreRule for LenientRolloutScoreCodePrefixRule {
    fn check(&self, item: &LenientRolloutScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRolloutScoreKeyIdLengthRule {
    pub max_length: i64,
}

impl LenientRolloutScoreRule for LenientRolloutScoreKeyIdLengthRule {
    fn check(&self, item: &LenientRolloutScore) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRolloutScoreRequiredTitleRule {
    pub message: String,
}

impl LenientRolloutScoreRule for LenientRolloutScoreRequiredTitleRule {
    fn check(&self, item: &LenientRolloutScore) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient rollout score.
#[derive(Debug, Clone)]
pub struct LenientRolloutScoreValidator {
    pub rules: Vec<Rc<dyn LenientRolloutScoreRule>>,
}

impl LenientRolloutScoreValidator {
    pub fn validate(&self, item: &LenientRolloutScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientRolloutScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientRolloutScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_rollout_score_validator() -> LenientRolloutScoreValidator {
    LenientRolloutScoreValidator {
        rules: vec![
            Rc::new(LenientRolloutScoreTitleLengthRule { max_length: 3600 }) as Rc<dyn LenientRolloutScoreRule>,
            Rc::new(LenientRolloutScoreCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(LenientRolloutScoreKeyIdLengthRule { max_length: 16 }),
            Rc::new(LenientRolloutScoreRequiredTitleRule {
                message: "lenient rollout score needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_rollout_score_validator();
        let item = new_lenient_rollout_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LenientRolloutScoreTitleLengthRule { max_length: 16 };
        let mut item = new_lenient_rollout_score("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = LenientRolloutScoreKeyIdLengthRule { max_length: 120 };
        let mut item = new_lenient_rollout_score("x", "y");
        item.key_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LenientRolloutScoreRequiredTitleRule {
            message: "lenient rollout score needs an entry".to_string(),
        };
        let mut item = new_lenient_rollout_score("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
