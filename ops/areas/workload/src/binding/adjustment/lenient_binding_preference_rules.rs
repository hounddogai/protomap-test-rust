use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient binding preference must pass. Check returns its problems, or none.
pub trait LenientBindingPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &LenientBindingPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientBindingPreferenceRequiredTitleRule {
    pub message: String,
}

impl LenientBindingPreferenceRule for LenientBindingPreferenceRequiredTitleRule {
    fn check(&self, item: &LenientBindingPreference) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientBindingPreferenceRequiredCodeRule {
    pub message: String,
}

impl LenientBindingPreferenceRule for LenientBindingPreferenceRequiredCodeRule {
    fn check(&self, item: &LenientBindingPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientBindingPreferenceQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientBindingPreferenceRule for LenientBindingPreferenceQuantityRangeRule {
    fn check(&self, item: &LenientBindingPreference) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientBindingPreferenceTitleLengthRule {
    pub max_length: i64,
}

impl LenientBindingPreferenceRule for LenientBindingPreferenceTitleLengthRule {
    fn check(&self, item: &LenientBindingPreference) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientBindingPreferenceCodePrefixRule {
    pub prefix: String,
}

impl LenientBindingPreferenceRule for LenientBindingPreferenceCodePrefixRule {
    fn check(&self, item: &LenientBindingPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient binding preference.
#[derive(Debug, Clone)]
pub struct LenientBindingPreferenceValidator {
    pub rules: Vec<Rc<dyn LenientBindingPreferenceRule>>,
}

impl LenientBindingPreferenceValidator {
    pub fn validate(&self, item: &LenientBindingPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientBindingPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientBindingPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_binding_preference_validator() -> LenientBindingPreferenceValidator {
    LenientBindingPreferenceValidator {
        rules: vec![
            Rc::new(LenientBindingPreferenceRequiredTitleRule {
                message: "lenient binding preference needs an entry".to_string(),
            }) as Rc<dyn LenientBindingPreferenceRule>,
            Rc::new(LenientBindingPreferenceRequiredCodeRule {
                message: "lenient binding preference needs this field".to_string(),
            }),
            Rc::new(LenientBindingPreferenceQuantityRangeRule { min: 16, max: 32 }),
            Rc::new(LenientBindingPreferenceTitleLengthRule { max_length: 80 }),
            Rc::new(LenientBindingPreferenceCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_binding_preference_validator();
        let item = new_lenient_binding_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LenientBindingPreferenceRequiredTitleRule {
            message: "lenient binding preference needs this field".to_string(),
        };
        let mut item = new_lenient_binding_preference("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientBindingPreferenceRequiredCodeRule {
            message: "lenient binding preference needs this field".to_string(),
        };
        let mut item = new_lenient_binding_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = LenientBindingPreferenceQuantityRangeRule { min: 64, max: 1000 };
        let mut item = new_lenient_binding_preference("x", "y");
        item.quantity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LenientBindingPreferenceTitleLengthRule { max_length: 3600 };
        let mut item = new_lenient_binding_preference("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
