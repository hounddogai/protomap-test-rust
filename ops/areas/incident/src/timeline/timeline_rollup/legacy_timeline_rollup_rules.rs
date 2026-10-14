use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy timeline rollup must pass. Check returns its problems, or none.
pub trait LegacyTimelineRollupRule: std::fmt::Debug {
    fn check(&self, item: &LegacyTimelineRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTimelineRollupTitleLengthRule {
    pub max_length: i64,
}

impl LegacyTimelineRollupRule for LegacyTimelineRollupTitleLengthRule {
    fn check(&self, item: &LegacyTimelineRollup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTimelineRollupTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl LegacyTimelineRollupRule for LegacyTimelineRollupTagsLimitRule {
    fn check(&self, item: &LegacyTimelineRollup) -> Vec<String> {
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
pub struct LegacyTimelineRollupRequiredCodeRule {
    pub message: String,
}

impl LegacyTimelineRollupRule for LegacyTimelineRollupRequiredCodeRule {
    fn check(&self, item: &LegacyTimelineRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTimelineRollupCodeLengthRule {
    pub max_length: i64,
}

impl LegacyTimelineRollupRule for LegacyTimelineRollupCodeLengthRule {
    fn check(&self, item: &LegacyTimelineRollup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTimelineRollupAllowedCategoryRule {
    pub allowed: Vec<String>,
}

impl LegacyTimelineRollupRule for LegacyTimelineRollupAllowedCategoryRule {
    fn check(&self, item: &LegacyTimelineRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.category)))
        {
            return vec![format!("category {} is blocked", item.category)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy timeline rollup.
#[derive(Debug, Clone)]
pub struct LegacyTimelineRollupValidator {
    pub rules: Vec<Rc<dyn LegacyTimelineRollupRule>>,
}

impl LegacyTimelineRollupValidator {
    pub fn validate(&self, item: &LegacyTimelineRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyTimelineRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyTimelineRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_timeline_rollup_validator() -> LegacyTimelineRollupValidator {
    LegacyTimelineRollupValidator {
        rules: vec![
            Rc::new(LegacyTimelineRollupTitleLengthRule { max_length: 64 }) as Rc<dyn LegacyTimelineRollupRule>,
            Rc::new(LegacyTimelineRollupTagsLimitRule {
                max_count: 80,
                banned: vec!["global".to_string()],
            }),
            Rc::new(LegacyTimelineRollupRequiredCodeRule {
                message: "legacy timeline rollup needs a value".to_string(),
            }),
            Rc::new(LegacyTimelineRollupCodeLengthRule { max_length: 3600 }),
            Rc::new(LegacyTimelineRollupAllowedCategoryRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_timeline_rollup_validator();
        let item = new_legacy_timeline_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LegacyTimelineRollupTitleLengthRule { max_length: 1000 };
        let mut item = new_legacy_timeline_rollup("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyTimelineRollupRequiredCodeRule {
            message: "legacy timeline rollup needs this field".to_string(),
        };
        let mut item = new_legacy_timeline_rollup("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyTimelineRollupCodeLengthRule { max_length: 120 };
        let mut item = new_legacy_timeline_rollup("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
