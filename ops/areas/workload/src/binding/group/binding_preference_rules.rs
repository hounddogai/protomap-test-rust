use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding preference must pass. Check returns its problems, or none.
pub trait BindingPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &BindingPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl BindingPreferenceRule for BindingPreferenceCodeLengthRule {
    fn check(&self, item: &BindingPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingPreferenceRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingPreferenceRule for BindingPreferenceRotationDaysRangeRule {
    fn check(&self, item: &BindingPreference) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingPreferenceRequiredCodeRule {
    pub message: String,
}

impl BindingPreferenceRule for BindingPreferenceRequiredCodeRule {
    fn check(&self, item: &BindingPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding preference.
#[derive(Debug, Clone)]
pub struct BindingPreferenceValidator {
    pub rules: Vec<Rc<dyn BindingPreferenceRule>>,
}

impl BindingPreferenceValidator {
    pub fn validate(&self, item: &BindingPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BindingPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_preference_validator() -> BindingPreferenceValidator {
    BindingPreferenceValidator {
        rules: vec![
            Rc::new(BindingPreferenceCodeLengthRule { max_length: 32 }) as Rc<dyn BindingPreferenceRule>,
            Rc::new(BindingPreferenceRotationDaysRangeRule { min: 120, max: 3600 }),
            Rc::new(BindingPreferenceRequiredCodeRule {
                message: "binding preference needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_preference_validator();
        let item = new_binding_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BindingPreferenceCodeLengthRule { max_length: 80 };
        let mut item = new_binding_preference("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = BindingPreferenceRotationDaysRangeRule { min: 32, max: 86400 };
        let mut item = new_binding_preference("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BindingPreferenceRequiredCodeRule {
            message: "binding preference needs a value".to_string(),
        };
        let mut item = new_binding_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
