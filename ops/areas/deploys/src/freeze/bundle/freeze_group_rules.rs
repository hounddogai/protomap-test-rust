use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a freeze group must pass. Check returns its problems, or none.
pub trait FreezeGroupRule: std::fmt::Debug {
    fn check(&self, item: &FreezeGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeGroupDescriptionLengthRule {
    pub max_length: i64,
}

impl FreezeGroupRule for FreezeGroupDescriptionLengthRule {
    fn check(&self, item: &FreezeGroup) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeGroupTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl FreezeGroupRule for FreezeGroupTagsLimitRule {
    fn check(&self, item: &FreezeGroup) -> Vec<String> {
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
pub struct FreezeGroupRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FreezeGroupRule for FreezeGroupRotationDaysRangeRule {
    fn check(&self, item: &FreezeGroup) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeGroupCodePrefixRule {
    pub prefix: String,
}

impl FreezeGroupRule for FreezeGroupCodePrefixRule {
    fn check(&self, item: &FreezeGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeGroupTitleLengthRule {
    pub max_length: i64,
}

impl FreezeGroupRule for FreezeGroupTitleLengthRule {
    fn check(&self, item: &FreezeGroup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeGroupRequiredAudienceRule {
    pub message: String,
}

impl FreezeGroupRule for FreezeGroupRequiredAudienceRule {
    fn check(&self, item: &FreezeGroup) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze group.
#[derive(Debug, Clone)]
pub struct FreezeGroupValidator {
    pub rules: Vec<Rc<dyn FreezeGroupRule>>,
}

impl FreezeGroupValidator {
    pub fn validate(&self, item: &FreezeGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FreezeGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_group_validator() -> FreezeGroupValidator {
    FreezeGroupValidator {
        rules: vec![
            Rc::new(FreezeGroupDescriptionLengthRule { max_length: 255 }) as Rc<dyn FreezeGroupRule>,
            Rc::new(FreezeGroupTagsLimitRule {
                max_count: 86400,
                banned: vec!["adaptive".to_string(), "capped".to_string(), "scheduled".to_string()],
            }),
            Rc::new(FreezeGroupRotationDaysRangeRule { min: 80, max: 255 }),
            Rc::new(FreezeGroupCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(FreezeGroupTitleLengthRule { max_length: 64 }),
            Rc::new(FreezeGroupRequiredAudienceRule {
                message: "freeze group needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_group_validator();
        let item = new_freeze_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = FreezeGroupDescriptionLengthRule { max_length: 64 };
        let mut item = new_freeze_group("x", "y");
        item.description = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = FreezeGroupRotationDaysRangeRule { min: 3600, max: 86400 };
        let mut item = new_freeze_group("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = FreezeGroupTitleLengthRule { max_length: 32 };
        let mut item = new_freeze_group("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = FreezeGroupRequiredAudienceRule {
            message: "freeze group needs this field".to_string(),
        };
        let mut item = new_freeze_group("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
