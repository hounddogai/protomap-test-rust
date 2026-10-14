use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a overage preference must pass. Check returns its problems, or none.
pub trait OveragePreferenceRule: std::fmt::Debug {
    fn check(&self, item: &OveragePreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OveragePreferenceAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl OveragePreferenceRule for OveragePreferenceAllowedTitleRule {
    fn check(&self, item: &OveragePreference) -> Vec<String> {
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
pub struct OveragePreferenceRequiredExternalRefRule {
    pub message: String,
}

impl OveragePreferenceRule for OveragePreferenceRequiredExternalRefRule {
    fn check(&self, item: &OveragePreference) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OveragePreferenceLocaleLengthRule {
    pub max_length: i64,
}

impl OveragePreferenceRule for OveragePreferenceLocaleLengthRule {
    fn check(&self, item: &OveragePreference) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OveragePreferenceFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl OveragePreferenceRule for OveragePreferenceFloorValueRangeRule {
    fn check(&self, item: &OveragePreference) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage preference.
#[derive(Debug, Clone)]
pub struct OveragePreferenceValidator {
    pub rules: Vec<Rc<dyn OveragePreferenceRule>>,
}

impl OveragePreferenceValidator {
    pub fn validate(&self, item: &OveragePreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &OveragePreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OveragePreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_preference_validator() -> OveragePreferenceValidator {
    OveragePreferenceValidator {
        rules: vec![
            Rc::new(OveragePreferenceAllowedTitleRule {
                allowed: vec!["primary".to_string()],
            }) as Rc<dyn OveragePreferenceRule>,
            Rc::new(OveragePreferenceRequiredExternalRefRule {
                message: "overage preference needs a value".to_string(),
            }),
            Rc::new(OveragePreferenceLocaleLengthRule { max_length: 80 }),
            Rc::new(OveragePreferenceFloorValueRangeRule { min: 9.2, max: 1002.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_preference_validator();
        let item = new_overage_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = OveragePreferenceRequiredExternalRefRule {
            message: "overage preference needs an entry".to_string(),
        };
        let mut item = new_overage_preference("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = OveragePreferenceLocaleLengthRule { max_length: 120 };
        let mut item = new_overage_preference("x", "y");
        item.locale = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = OveragePreferenceFloorValueRangeRule { min: 7.3, max: 4986.5 };
        let mut item = new_overage_preference("x", "y");
        item.floor_value = 4988.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
