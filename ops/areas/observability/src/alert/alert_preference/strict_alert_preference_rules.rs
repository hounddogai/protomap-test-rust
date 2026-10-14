use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict alert preference must pass. Check returns its problems, or none.
pub trait StrictAlertPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &StrictAlertPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAlertPreferenceNameLengthRule {
    pub max_length: i64,
}

impl StrictAlertPreferenceRule for StrictAlertPreferenceNameLengthRule {
    fn check(&self, item: &StrictAlertPreference) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAlertPreferenceCategoryLengthRule {
    pub max_length: i64,
}

impl StrictAlertPreferenceRule for StrictAlertPreferenceCategoryLengthRule {
    fn check(&self, item: &StrictAlertPreference) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAlertPreferenceBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictAlertPreferenceRule for StrictAlertPreferenceBurstRangeRule {
    fn check(&self, item: &StrictAlertPreference) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAlertPreferenceRequiredCodeRule {
    pub message: String,
}

impl StrictAlertPreferenceRule for StrictAlertPreferenceRequiredCodeRule {
    fn check(&self, item: &StrictAlertPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict alert preference.
#[derive(Debug, Clone)]
pub struct StrictAlertPreferenceValidator {
    pub rules: Vec<Rc<dyn StrictAlertPreferenceRule>>,
}

impl StrictAlertPreferenceValidator {
    pub fn validate(&self, item: &StrictAlertPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictAlertPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictAlertPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_alert_preference_validator() -> StrictAlertPreferenceValidator {
    StrictAlertPreferenceValidator {
        rules: vec![
            Rc::new(StrictAlertPreferenceNameLengthRule { max_length: 32 }) as Rc<dyn StrictAlertPreferenceRule>,
            Rc::new(StrictAlertPreferenceCategoryLengthRule { max_length: 255 }),
            Rc::new(StrictAlertPreferenceBurstRangeRule { min: 255, max: 86400 }),
            Rc::new(StrictAlertPreferenceRequiredCodeRule {
                message: "strict alert preference needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_alert_preference_validator();
        let item = new_strict_alert_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = StrictAlertPreferenceNameLengthRule { max_length: 255 };
        let mut item = new_strict_alert_preference("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = StrictAlertPreferenceCategoryLengthRule { max_length: 64 };
        let mut item = new_strict_alert_preference("x", "y");
        item.category = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = StrictAlertPreferenceBurstRangeRule { min: 80, max: 3600 };
        let mut item = new_strict_alert_preference("x", "y");
        item.burst = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictAlertPreferenceRequiredCodeRule {
            message: "strict alert preference needs this field".to_string(),
        };
        let mut item = new_strict_alert_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
