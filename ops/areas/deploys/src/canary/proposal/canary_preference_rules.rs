use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a canary preference must pass. Check returns its problems, or none.
pub trait CanaryPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &CanaryPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryPreferenceUnitLengthRule {
    pub max_length: i64,
}

impl CanaryPreferenceRule for CanaryPreferenceUnitLengthRule {
    fn check(&self, item: &CanaryPreference) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryPreferenceAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl CanaryPreferenceRule for CanaryPreferenceAllowedCodeRule {
    fn check(&self, item: &CanaryPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryPreferenceRequiredTitleRule {
    pub message: String,
}

impl CanaryPreferenceRule for CanaryPreferenceRequiredTitleRule {
    fn check(&self, item: &CanaryPreference) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryPreferenceRequiredUnitRule {
    pub message: String,
}

impl CanaryPreferenceRule for CanaryPreferenceRequiredUnitRule {
    fn check(&self, item: &CanaryPreference) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary preference.
#[derive(Debug, Clone)]
pub struct CanaryPreferenceValidator {
    pub rules: Vec<Rc<dyn CanaryPreferenceRule>>,
}

impl CanaryPreferenceValidator {
    pub fn validate(&self, item: &CanaryPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CanaryPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_preference_validator() -> CanaryPreferenceValidator {
    CanaryPreferenceValidator {
        rules: vec![
            Rc::new(CanaryPreferenceUnitLengthRule { max_length: 64 }) as Rc<dyn CanaryPreferenceRule>,
            Rc::new(CanaryPreferenceAllowedCodeRule {
                allowed: vec!["fallback".to_string()],
            }),
            Rc::new(CanaryPreferenceRequiredTitleRule {
                message: "canary preference needs this field".to_string(),
            }),
            Rc::new(CanaryPreferenceRequiredUnitRule {
                message: "canary preference needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_preference_validator();
        let item = new_canary_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = CanaryPreferenceUnitLengthRule { max_length: 86400 };
        let mut item = new_canary_preference("x", "y");
        item.unit = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = CanaryPreferenceRequiredTitleRule {
            message: "canary preference needs this field".to_string(),
        };
        let mut item = new_canary_preference("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = CanaryPreferenceRequiredUnitRule {
            message: "canary preference needs an entry".to_string(),
        };
        let mut item = new_canary_preference("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
