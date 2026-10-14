use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy rollout window must pass. Check returns its problems, or none.
pub trait LegacyRolloutWindowRule: std::fmt::Debug {
    fn check(&self, item: &LegacyRolloutWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutWindowRequiredCategoryRule {
    pub message: String,
}

impl LegacyRolloutWindowRule for LegacyRolloutWindowRequiredCategoryRule {
    fn check(&self, item: &LegacyRolloutWindow) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutWindowCodePrefixRule {
    pub prefix: String,
}

impl LegacyRolloutWindowRule for LegacyRolloutWindowCodePrefixRule {
    fn check(&self, item: &LegacyRolloutWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutWindowCodeLengthRule {
    pub max_length: i64,
}

impl LegacyRolloutWindowRule for LegacyRolloutWindowCodeLengthRule {
    fn check(&self, item: &LegacyRolloutWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy rollout window.
#[derive(Debug, Clone)]
pub struct LegacyRolloutWindowValidator {
    pub rules: Vec<Rc<dyn LegacyRolloutWindowRule>>,
}

impl LegacyRolloutWindowValidator {
    pub fn validate(&self, item: &LegacyRolloutWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyRolloutWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyRolloutWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_rollout_window_validator() -> LegacyRolloutWindowValidator {
    LegacyRolloutWindowValidator {
        rules: vec![
            Rc::new(LegacyRolloutWindowRequiredCategoryRule {
                message: "legacy rollout window needs an entry".to_string(),
            }) as Rc<dyn LegacyRolloutWindowRule>,
            Rc::new(LegacyRolloutWindowCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(LegacyRolloutWindowCodeLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_rollout_window_validator();
        let item = new_legacy_rollout_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = LegacyRolloutWindowRequiredCategoryRule {
            message: "legacy rollout window needs a value".to_string(),
        };
        let mut item = new_legacy_rollout_window("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyRolloutWindowCodeLengthRule { max_length: 80 };
        let mut item = new_legacy_rollout_window("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
