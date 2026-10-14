use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin preference must pass. Check returns its problems, or none.
pub trait RegionPinPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinPreferenceCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionPinPreferenceRule for RegionPinPreferenceCreatedAtRangeRule {
    fn check(&self, item: &RegionPinPreference) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinPreferenceTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl RegionPinPreferenceRule for RegionPinPreferenceTagsLimitRule {
    fn check(&self, item: &RegionPinPreference) -> Vec<String> {
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
pub struct RegionPinPreferenceRequiredCodeRule {
    pub message: String,
}

impl RegionPinPreferenceRule for RegionPinPreferenceRequiredCodeRule {
    fn check(&self, item: &RegionPinPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl RegionPinPreferenceRule for RegionPinPreferenceCodeLengthRule {
    fn check(&self, item: &RegionPinPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin preference.
#[derive(Debug, Clone)]
pub struct RegionPinPreferenceValidator {
    pub rules: Vec<Rc<dyn RegionPinPreferenceRule>>,
}

impl RegionPinPreferenceValidator {
    pub fn validate(&self, item: &RegionPinPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionPinPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_preference_validator() -> RegionPinPreferenceValidator {
    RegionPinPreferenceValidator {
        rules: vec![
            Rc::new(RegionPinPreferenceCreatedAtRangeRule { min: 32, max: 1000 }) as Rc<dyn RegionPinPreferenceRule>,
            Rc::new(RegionPinPreferenceTagsLimitRule {
                max_count: 86400,
                banned: vec!["regional".to_string(), "rolling".to_string()],
            }),
            Rc::new(RegionPinPreferenceRequiredCodeRule {
                message: "region pin preference needs an entry".to_string(),
            }),
            Rc::new(RegionPinPreferenceCodeLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_preference_validator();
        let item = new_region_pin_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = RegionPinPreferenceCreatedAtRangeRule { min: 64, max: 3600 };
        let mut item = new_region_pin_preference("x", "y");
        item.created_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionPinPreferenceRequiredCodeRule {
            message: "region pin preference needs a value".to_string(),
        };
        let mut item = new_region_pin_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionPinPreferenceCodeLengthRule { max_length: 86400 };
        let mut item = new_region_pin_preference("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
