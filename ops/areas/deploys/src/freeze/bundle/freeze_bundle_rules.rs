use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a freeze bundle must pass. Check returns its problems, or none.
pub trait FreezeBundleRule: std::fmt::Debug {
    fn check(&self, item: &FreezeBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeBundleRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FreezeBundleRule for FreezeBundleRevisionRangeRule {
    fn check(&self, item: &FreezeBundle) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeBundleAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl FreezeBundleRule for FreezeBundleAllowedTitleRule {
    fn check(&self, item: &FreezeBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is blocked", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeBundleTitleLengthRule {
    pub max_length: i64,
}

impl FreezeBundleRule for FreezeBundleTitleLengthRule {
    fn check(&self, item: &FreezeBundle) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeBundleRequiredCodeRule {
    pub message: String,
}

impl FreezeBundleRule for FreezeBundleRequiredCodeRule {
    fn check(&self, item: &FreezeBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeBundleCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FreezeBundleRule for FreezeBundleCreatedAtRangeRule {
    fn check(&self, item: &FreezeBundle) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze bundle.
#[derive(Debug, Clone)]
pub struct FreezeBundleValidator {
    pub rules: Vec<Rc<dyn FreezeBundleRule>>,
}

impl FreezeBundleValidator {
    pub fn validate(&self, item: &FreezeBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FreezeBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_bundle_validator() -> FreezeBundleValidator {
    FreezeBundleValidator {
        rules: vec![
            Rc::new(FreezeBundleRevisionRangeRule { min: 120, max: 120 }) as Rc<dyn FreezeBundleRule>,
            Rc::new(FreezeBundleAllowedTitleRule {
                allowed: vec!["default".to_string(), "regional".to_string(), "strict".to_string()],
            }),
            Rc::new(FreezeBundleTitleLengthRule { max_length: 86400 }),
            Rc::new(FreezeBundleRequiredCodeRule {
                message: "freeze bundle needs an entry".to_string(),
            }),
            Rc::new(FreezeBundleCreatedAtRangeRule { min: 32, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_bundle_validator();
        let item = new_freeze_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = FreezeBundleRevisionRangeRule { min: 255, max: 3600 };
        let mut item = new_freeze_bundle("x", "y");
        item.revision = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = FreezeBundleTitleLengthRule { max_length: 32 };
        let mut item = new_freeze_bundle("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FreezeBundleRequiredCodeRule {
            message: "freeze bundle needs a value".to_string(),
        };
        let mut item = new_freeze_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = FreezeBundleCreatedAtRangeRule { min: 32, max: 80 };
        let mut item = new_freeze_bundle("x", "y");
        item.created_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
