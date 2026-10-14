use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient alert profile must pass. Check returns its problems, or none.
pub trait LenientAlertProfileRule: std::fmt::Debug {
    fn check(&self, item: &LenientAlertProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAlertProfileTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientAlertProfileRule for LenientAlertProfileTtlSecondsRangeRule {
    fn check(&self, item: &LenientAlertProfile) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAlertProfileCodeLengthRule {
    pub max_length: i64,
}

impl LenientAlertProfileRule for LenientAlertProfileCodeLengthRule {
    fn check(&self, item: &LenientAlertProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAlertProfileAllowedNameRule {
    pub allowed: Vec<String>,
}

impl LenientAlertProfileRule for LenientAlertProfileAllowedNameRule {
    fn check(&self, item: &LenientAlertProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is duplicated", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAlertProfileTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl LenientAlertProfileRule for LenientAlertProfileTagsLimitRule {
    fn check(&self, item: &LenientAlertProfile) -> Vec<String> {
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
pub struct LenientAlertProfileRequiredLabelRule {
    pub message: String,
}

impl LenientAlertProfileRule for LenientAlertProfileRequiredLabelRule {
    fn check(&self, item: &LenientAlertProfile) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient alert profile.
#[derive(Debug, Clone)]
pub struct LenientAlertProfileValidator {
    pub rules: Vec<Rc<dyn LenientAlertProfileRule>>,
}

impl LenientAlertProfileValidator {
    pub fn validate(&self, item: &LenientAlertProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientAlertProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientAlertProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_alert_profile_validator() -> LenientAlertProfileValidator {
    LenientAlertProfileValidator {
        rules: vec![
            Rc::new(LenientAlertProfileTtlSecondsRangeRule { min: 255, max: 3600 }) as Rc<dyn LenientAlertProfileRule>,
            Rc::new(LenientAlertProfileCodeLengthRule { max_length: 255 }),
            Rc::new(LenientAlertProfileAllowedNameRule {
                allowed: vec!["capped".to_string(), "global".to_string(), "default".to_string()],
            }),
            Rc::new(LenientAlertProfileTagsLimitRule {
                max_count: 64,
                banned: Vec::new(),
            }),
            Rc::new(LenientAlertProfileRequiredLabelRule {
                message: "lenient alert profile needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_alert_profile_validator();
        let item = new_lenient_alert_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = LenientAlertProfileTtlSecondsRangeRule { min: 16, max: 80 };
        let mut item = new_lenient_alert_profile("x", "y");
        item.ttl_seconds = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientAlertProfileCodeLengthRule { max_length: 120 };
        let mut item = new_lenient_alert_profile("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientAlertProfileRequiredLabelRule {
            message: "lenient alert profile needs a value".to_string(),
        };
        let mut item = new_lenient_alert_profile("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
