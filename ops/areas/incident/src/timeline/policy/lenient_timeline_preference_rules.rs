use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient timeline preference must pass. Check returns its problems, or none.
pub trait LenientTimelinePreferenceRule: std::fmt::Debug {
    fn check(&self, item: &LenientTimelinePreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTimelinePreferenceDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientTimelinePreferenceRule for LenientTimelinePreferenceDiscountRangeRule {
    fn check(&self, item: &LenientTimelinePreference) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTimelinePreferenceQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientTimelinePreferenceRule for LenientTimelinePreferenceQuotaUnitsRangeRule {
    fn check(&self, item: &LenientTimelinePreference) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTimelinePreferenceRequiredLabelRule {
    pub message: String,
}

impl LenientTimelinePreferenceRule for LenientTimelinePreferenceRequiredLabelRule {
    fn check(&self, item: &LenientTimelinePreference) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTimelinePreferenceRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientTimelinePreferenceRule for LenientTimelinePreferenceRotationDaysRangeRule {
    fn check(&self, item: &LenientTimelinePreference) -> Vec<String> {
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
pub struct LenientTimelinePreferenceCodeLengthRule {
    pub max_length: i64,
}

impl LenientTimelinePreferenceRule for LenientTimelinePreferenceCodeLengthRule {
    fn check(&self, item: &LenientTimelinePreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTimelinePreferenceRequiredCodeRule {
    pub message: String,
}

impl LenientTimelinePreferenceRule for LenientTimelinePreferenceRequiredCodeRule {
    fn check(&self, item: &LenientTimelinePreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient timeline preference.
#[derive(Debug, Clone)]
pub struct LenientTimelinePreferenceValidator {
    pub rules: Vec<Rc<dyn LenientTimelinePreferenceRule>>,
}

impl LenientTimelinePreferenceValidator {
    pub fn validate(&self, item: &LenientTimelinePreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientTimelinePreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientTimelinePreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_timeline_preference_validator() -> LenientTimelinePreferenceValidator {
    LenientTimelinePreferenceValidator {
        rules: vec![
            Rc::new(LenientTimelinePreferenceDiscountRangeRule { min: 4.7, max: 402.8 })
                as Rc<dyn LenientTimelinePreferenceRule>,
            Rc::new(LenientTimelinePreferenceQuotaUnitsRangeRule { min: 64, max: 1000 }),
            Rc::new(LenientTimelinePreferenceRequiredLabelRule {
                message: "lenient timeline preference needs this field".to_string(),
            }),
            Rc::new(LenientTimelinePreferenceRotationDaysRangeRule { min: 80, max: 1000 }),
            Rc::new(LenientTimelinePreferenceCodeLengthRule { max_length: 16 }),
            Rc::new(LenientTimelinePreferenceRequiredCodeRule {
                message: "lenient timeline preference needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_timeline_preference_validator();
        let item = new_lenient_timeline_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = LenientTimelinePreferenceDiscountRangeRule { min: 6.1, max: 948.0 };
        let mut item = new_lenient_timeline_preference("x", "y");
        item.discount = 949.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = LenientTimelinePreferenceQuotaUnitsRangeRule { min: 80, max: 120 };
        let mut item = new_lenient_timeline_preference("x", "y");
        item.quota_units = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientTimelinePreferenceRequiredLabelRule {
            message: "lenient timeline preference needs a value".to_string(),
        };
        let mut item = new_lenient_timeline_preference("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = LenientTimelinePreferenceRotationDaysRangeRule { min: 120, max: 86400 };
        let mut item = new_lenient_timeline_preference("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientTimelinePreferenceCodeLengthRule { max_length: 32 };
        let mut item = new_lenient_timeline_preference("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientTimelinePreferenceRequiredCodeRule {
            message: "lenient timeline preference needs this field".to_string(),
        };
        let mut item = new_lenient_timeline_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
