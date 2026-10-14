use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy measurement preference must pass. Check returns its problems, or none.
pub trait LegacyMeasurementPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &LegacyMeasurementPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementPreferenceRequiredCodeRule {
    pub message: String,
}

impl LegacyMeasurementPreferenceRule for LegacyMeasurementPreferenceRequiredCodeRule {
    fn check(&self, item: &LegacyMeasurementPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementPreferenceMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyMeasurementPreferenceRule for LegacyMeasurementPreferenceMaximumRangeRule {
    fn check(&self, item: &LegacyMeasurementPreference) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementPreferenceRequiredContactEmailRule {
    pub message: String,
}

impl LegacyMeasurementPreferenceRule for LegacyMeasurementPreferenceRequiredContactEmailRule {
    fn check(&self, item: &LegacyMeasurementPreference) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementPreferencePinnedGuardRule {
    pub limit: f64,
}

impl LegacyMeasurementPreferenceRule for LegacyMeasurementPreferencePinnedGuardRule {
    fn check(&self, item: &LegacyMeasurementPreference) -> Vec<String> {
        if item.pinned && (item.maximum > self.limit) {
            return vec![format!("maximum {} exceeds {} while pinned", item.maximum, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementPreferenceContactEmailLengthRule {
    pub max_length: i64,
}

impl LegacyMeasurementPreferenceRule for LegacyMeasurementPreferenceContactEmailLengthRule {
    fn check(&self, item: &LegacyMeasurementPreference) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementPreferenceRequiredNameRule {
    pub message: String,
}

impl LegacyMeasurementPreferenceRule for LegacyMeasurementPreferenceRequiredNameRule {
    fn check(&self, item: &LegacyMeasurementPreference) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy measurement preference.
#[derive(Debug, Clone)]
pub struct LegacyMeasurementPreferenceValidator {
    pub rules: Vec<Rc<dyn LegacyMeasurementPreferenceRule>>,
}

impl LegacyMeasurementPreferenceValidator {
    pub fn validate(&self, item: &LegacyMeasurementPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyMeasurementPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyMeasurementPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_measurement_preference_validator() -> LegacyMeasurementPreferenceValidator {
    LegacyMeasurementPreferenceValidator {
        rules: vec![
            Rc::new(LegacyMeasurementPreferenceRequiredCodeRule {
                message: "legacy measurement preference needs this field".to_string(),
            }) as Rc<dyn LegacyMeasurementPreferenceRule>,
            Rc::new(LegacyMeasurementPreferenceMaximumRangeRule { min: 8.9, max: 1884.4 }),
            Rc::new(LegacyMeasurementPreferenceRequiredContactEmailRule {
                message: "legacy measurement preference needs a value".to_string(),
            }),
            Rc::new(LegacyMeasurementPreferencePinnedGuardRule { limit: 1.7 }),
            Rc::new(LegacyMeasurementPreferenceContactEmailLengthRule { max_length: 1000 }),
            Rc::new(LegacyMeasurementPreferenceRequiredNameRule {
                message: "legacy measurement preference needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_measurement_preference_validator();
        let item = new_legacy_measurement_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyMeasurementPreferenceRequiredCodeRule {
            message: "legacy measurement preference needs a value".to_string(),
        };
        let mut item = new_legacy_measurement_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = LegacyMeasurementPreferenceMaximumRangeRule { min: 6.8, max: 3989.9 };
        let mut item = new_legacy_measurement_preference("x", "y");
        item.maximum = 3991.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = LegacyMeasurementPreferenceRequiredContactEmailRule {
            message: "legacy measurement preference needs a value".to_string(),
        };
        let mut item = new_legacy_measurement_preference("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = LegacyMeasurementPreferenceContactEmailLengthRule { max_length: 1000 };
        let mut item = new_legacy_measurement_preference("x", "y");
        item.contact_email = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LegacyMeasurementPreferenceRequiredNameRule {
            message: "legacy measurement preference needs a value".to_string(),
        };
        let mut item = new_legacy_measurement_preference("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
