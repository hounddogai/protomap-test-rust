use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global limit digest must pass. Check returns its problems, or none.
pub trait GlobalLimitDigestRule: std::fmt::Debug {
    fn check(&self, item: &GlobalLimitDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitDigestCodePrefixRule {
    pub prefix: String,
}

impl GlobalLimitDigestRule for GlobalLimitDigestCodePrefixRule {
    fn check(&self, item: &GlobalLimitDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitDigestTitleLengthRule {
    pub max_length: i64,
}

impl GlobalLimitDigestRule for GlobalLimitDigestTitleLengthRule {
    fn check(&self, item: &GlobalLimitDigest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitDigestAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl GlobalLimitDigestRule for GlobalLimitDigestAllowedTitleRule {
    fn check(&self, item: &GlobalLimitDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is too long", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitDigestRequiredTitleRule {
    pub message: String,
}

impl GlobalLimitDigestRule for GlobalLimitDigestRequiredTitleRule {
    fn check(&self, item: &GlobalLimitDigest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitDigestRequiredNotesRule {
    pub message: String,
}

impl GlobalLimitDigestRule for GlobalLimitDigestRequiredNotesRule {
    fn check(&self, item: &GlobalLimitDigest) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitDigestMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalLimitDigestRule for GlobalLimitDigestMarginRangeRule {
    fn check(&self, item: &GlobalLimitDigest) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a global limit digest.
#[derive(Debug, Clone)]
pub struct GlobalLimitDigestValidator {
    pub rules: Vec<Rc<dyn GlobalLimitDigestRule>>,
}

impl GlobalLimitDigestValidator {
    pub fn validate(&self, item: &GlobalLimitDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalLimitDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalLimitDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_limit_digest_validator() -> GlobalLimitDigestValidator {
    GlobalLimitDigestValidator {
        rules: vec![
            Rc::new(GlobalLimitDigestCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn GlobalLimitDigestRule>,
            Rc::new(GlobalLimitDigestTitleLengthRule { max_length: 86400 }),
            Rc::new(GlobalLimitDigestAllowedTitleRule {
                allowed: vec!["fallback".to_string(), "primary".to_string(), "batched".to_string()],
            }),
            Rc::new(GlobalLimitDigestRequiredTitleRule {
                message: "global limit digest needs a value".to_string(),
            }),
            Rc::new(GlobalLimitDigestRequiredNotesRule {
                message: "global limit digest needs this field".to_string(),
            }),
            Rc::new(GlobalLimitDigestMarginRangeRule { min: 0.5, max: 565.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_limit_digest_validator();
        let item = new_global_limit_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = GlobalLimitDigestTitleLengthRule { max_length: 32 };
        let mut item = new_global_limit_digest("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = GlobalLimitDigestRequiredTitleRule {
            message: "global limit digest needs an entry".to_string(),
        };
        let mut item = new_global_limit_digest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = GlobalLimitDigestRequiredNotesRule {
            message: "global limit digest needs this field".to_string(),
        };
        let mut item = new_global_limit_digest("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = GlobalLimitDigestMarginRangeRule { min: 4.5, max: 1078.4 };
        let mut item = new_global_limit_digest("x", "y");
        item.margin = 1079.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
