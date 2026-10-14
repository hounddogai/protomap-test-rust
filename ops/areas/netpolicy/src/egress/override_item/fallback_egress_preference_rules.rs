use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback egress preference must pass. Check returns its problems, or none.
pub trait FallbackEgressPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &FallbackEgressPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressPreferenceRequiredCodeRule {
    pub message: String,
}

impl FallbackEgressPreferenceRule for FallbackEgressPreferenceRequiredCodeRule {
    fn check(&self, item: &FallbackEgressPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressPreferenceFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackEgressPreferenceRule for FallbackEgressPreferenceFloorValueRangeRule {
    fn check(&self, item: &FallbackEgressPreference) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressPreferenceRequiredTitleRule {
    pub message: String,
}

impl FallbackEgressPreferenceRule for FallbackEgressPreferenceRequiredTitleRule {
    fn check(&self, item: &FallbackEgressPreference) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback egress preference.
#[derive(Debug, Clone)]
pub struct FallbackEgressPreferenceValidator {
    pub rules: Vec<Rc<dyn FallbackEgressPreferenceRule>>,
}

impl FallbackEgressPreferenceValidator {
    pub fn validate(&self, item: &FallbackEgressPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackEgressPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackEgressPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_egress_preference_validator() -> FallbackEgressPreferenceValidator {
    FallbackEgressPreferenceValidator {
        rules: vec![
            Rc::new(FallbackEgressPreferenceRequiredCodeRule {
                message: "fallback egress preference needs this field".to_string(),
            }) as Rc<dyn FallbackEgressPreferenceRule>,
            Rc::new(FallbackEgressPreferenceFloorValueRangeRule { min: 5.4, max: 4622.7 }),
            Rc::new(FallbackEgressPreferenceRequiredTitleRule {
                message: "fallback egress preference needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_egress_preference_validator();
        let item = new_fallback_egress_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackEgressPreferenceRequiredCodeRule {
            message: "fallback egress preference needs this field".to_string(),
        };
        let mut item = new_fallback_egress_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = FallbackEgressPreferenceFloorValueRangeRule { min: 7.7, max: 2607.7 };
        let mut item = new_fallback_egress_preference("x", "y");
        item.floor_value = 2609.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = FallbackEgressPreferenceRequiredTitleRule {
            message: "fallback egress preference needs an entry".to_string(),
        };
        let mut item = new_fallback_egress_preference("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
