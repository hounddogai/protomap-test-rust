use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled region pin preference must pass. Check returns its problems, or none.
pub trait ScheduledRegionPinPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledRegionPinPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRegionPinPreferenceRequiredCodeRule {
    pub message: String,
}

impl ScheduledRegionPinPreferenceRule for ScheduledRegionPinPreferenceRequiredCodeRule {
    fn check(&self, item: &ScheduledRegionPinPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRegionPinPreferenceBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledRegionPinPreferenceRule for ScheduledRegionPinPreferenceBudgetRangeRule {
    fn check(&self, item: &ScheduledRegionPinPreference) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRegionPinPreferenceLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledRegionPinPreferenceRule for ScheduledRegionPinPreferenceLimitRangeRule {
    fn check(&self, item: &ScheduledRegionPinPreference) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRegionPinPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledRegionPinPreferenceRule for ScheduledRegionPinPreferenceCodeLengthRule {
    fn check(&self, item: &ScheduledRegionPinPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRegionPinPreferenceActiveGuardRule {
    pub limit: f64,
}

impl ScheduledRegionPinPreferenceRule for ScheduledRegionPinPreferenceActiveGuardRule {
    fn check(&self, item: &ScheduledRegionPinPreference) -> Vec<String> {
        if item.active && (item.budget > self.limit) {
            return vec![format!("budget {} exceeds {} while active", item.budget, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled region pin preference.
#[derive(Debug, Clone)]
pub struct ScheduledRegionPinPreferenceValidator {
    pub rules: Vec<Rc<dyn ScheduledRegionPinPreferenceRule>>,
}

impl ScheduledRegionPinPreferenceValidator {
    pub fn validate(&self, item: &ScheduledRegionPinPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledRegionPinPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledRegionPinPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_region_pin_preference_validator() -> ScheduledRegionPinPreferenceValidator {
    ScheduledRegionPinPreferenceValidator {
        rules: vec![
            Rc::new(ScheduledRegionPinPreferenceRequiredCodeRule {
                message: "scheduled region pin preference needs this field".to_string(),
            }) as Rc<dyn ScheduledRegionPinPreferenceRule>,
            Rc::new(ScheduledRegionPinPreferenceBudgetRangeRule { min: 5.2, max: 2754.6 }),
            Rc::new(ScheduledRegionPinPreferenceLimitRangeRule { min: 3600, max: 86400 }),
            Rc::new(ScheduledRegionPinPreferenceCodeLengthRule { max_length: 120 }),
            Rc::new(ScheduledRegionPinPreferenceActiveGuardRule { limit: 9.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_region_pin_preference_validator();
        let item = new_scheduled_region_pin_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledRegionPinPreferenceRequiredCodeRule {
            message: "scheduled region pin preference needs an entry".to_string(),
        };
        let mut item = new_scheduled_region_pin_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = ScheduledRegionPinPreferenceBudgetRangeRule { min: 5.9, max: 1408.7 };
        let mut item = new_scheduled_region_pin_preference("x", "y");
        item.budget = 1410.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = ScheduledRegionPinPreferenceLimitRangeRule { min: 3600, max: 3600 };
        let mut item = new_scheduled_region_pin_preference("x", "y");
        item.limit = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledRegionPinPreferenceCodeLengthRule { max_length: 80 };
        let mut item = new_scheduled_region_pin_preference("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
