use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal rollout preference must pass. Check returns its problems, or none.
pub trait SeasonalRolloutPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalRolloutPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRolloutPreferenceRequiredLabelRule {
    pub message: String,
}

impl SeasonalRolloutPreferenceRule for SeasonalRolloutPreferenceRequiredLabelRule {
    fn check(&self, item: &SeasonalRolloutPreference) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRolloutPreferenceTitleLengthRule {
    pub max_length: i64,
}

impl SeasonalRolloutPreferenceRule for SeasonalRolloutPreferenceTitleLengthRule {
    fn check(&self, item: &SeasonalRolloutPreference) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRolloutPreferenceAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl SeasonalRolloutPreferenceRule for SeasonalRolloutPreferenceAllowedCodeRule {
    fn check(&self, item: &SeasonalRolloutPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRolloutPreferenceCodePrefixRule {
    pub prefix: String,
}

impl SeasonalRolloutPreferenceRule for SeasonalRolloutPreferenceCodePrefixRule {
    fn check(&self, item: &SeasonalRolloutPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRolloutPreferencePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalRolloutPreferenceRule for SeasonalRolloutPreferencePortRangeRule {
    fn check(&self, item: &SeasonalRolloutPreference) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a seasonal rollout preference.
#[derive(Debug, Clone)]
pub struct SeasonalRolloutPreferenceValidator {
    pub rules: Vec<Rc<dyn SeasonalRolloutPreferenceRule>>,
}

impl SeasonalRolloutPreferenceValidator {
    pub fn validate(&self, item: &SeasonalRolloutPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalRolloutPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalRolloutPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_rollout_preference_validator() -> SeasonalRolloutPreferenceValidator {
    SeasonalRolloutPreferenceValidator {
        rules: vec![
            Rc::new(SeasonalRolloutPreferenceRequiredLabelRule {
                message: "seasonal rollout preference needs a value".to_string(),
            }) as Rc<dyn SeasonalRolloutPreferenceRule>,
            Rc::new(SeasonalRolloutPreferenceTitleLengthRule { max_length: 86400 }),
            Rc::new(SeasonalRolloutPreferenceAllowedCodeRule {
                allowed: vec!["batched".to_string()],
            }),
            Rc::new(SeasonalRolloutPreferenceCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(SeasonalRolloutPreferencePortRangeRule { min: 16, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_rollout_preference_validator();
        let item = new_seasonal_rollout_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SeasonalRolloutPreferenceRequiredLabelRule {
            message: "seasonal rollout preference needs this field".to_string(),
        };
        let mut item = new_seasonal_rollout_preference("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SeasonalRolloutPreferenceTitleLengthRule { max_length: 3600 };
        let mut item = new_seasonal_rollout_preference("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = SeasonalRolloutPreferencePortRangeRule { min: 16, max: 86400 };
        let mut item = new_seasonal_rollout_preference("x", "y");
        item.port = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
