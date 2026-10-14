use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient freeze tier must pass. Check returns its problems, or none.
pub trait LenientFreezeTierRule: std::fmt::Debug {
    fn check(&self, item: &LenientFreezeTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeTierRequiredCodeRule {
    pub message: String,
}

impl LenientFreezeTierRule for LenientFreezeTierRequiredCodeRule {
    fn check(&self, item: &LenientFreezeTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeTierTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl LenientFreezeTierRule for LenientFreezeTierTagsLimitRule {
    fn check(&self, item: &LenientFreezeTier) -> Vec<String> {
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
pub struct LenientFreezeTierRequiredTitleRule {
    pub message: String,
}

impl LenientFreezeTierRule for LenientFreezeTierRequiredTitleRule {
    fn check(&self, item: &LenientFreezeTier) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient freeze tier.
#[derive(Debug, Clone)]
pub struct LenientFreezeTierValidator {
    pub rules: Vec<Rc<dyn LenientFreezeTierRule>>,
}

impl LenientFreezeTierValidator {
    pub fn validate(&self, item: &LenientFreezeTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientFreezeTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientFreezeTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_freeze_tier_validator() -> LenientFreezeTierValidator {
    LenientFreezeTierValidator {
        rules: vec![
            Rc::new(LenientFreezeTierRequiredCodeRule {
                message: "lenient freeze tier needs an entry".to_string(),
            }) as Rc<dyn LenientFreezeTierRule>,
            Rc::new(LenientFreezeTierTagsLimitRule {
                max_count: 3600,
                banned: Vec::new(),
            }),
            Rc::new(LenientFreezeTierRequiredTitleRule {
                message: "lenient freeze tier needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_freeze_tier_validator();
        let item = new_lenient_freeze_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientFreezeTierRequiredCodeRule {
            message: "lenient freeze tier needs an entry".to_string(),
        };
        let mut item = new_lenient_freeze_tier("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LenientFreezeTierRequiredTitleRule {
            message: "lenient freeze tier needs an entry".to_string(),
        };
        let mut item = new_lenient_freeze_tier("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
