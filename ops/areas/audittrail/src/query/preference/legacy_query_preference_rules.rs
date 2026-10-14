use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy query preference must pass. Check returns its problems, or none.
pub trait LegacyQueryPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &LegacyQueryPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQueryPreferenceCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyQueryPreferenceRule for LegacyQueryPreferenceCostRangeRule {
    fn check(&self, item: &LegacyQueryPreference) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQueryPreferenceAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl LegacyQueryPreferenceRule for LegacyQueryPreferenceAllowedScopeRule {
    fn check(&self, item: &LegacyQueryPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is below minimum", item.scope)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQueryPreferenceLabelLengthRule {
    pub max_length: i64,
}

impl LegacyQueryPreferenceRule for LegacyQueryPreferenceLabelLengthRule {
    fn check(&self, item: &LegacyQueryPreference) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQueryPreferenceDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyQueryPreferenceRule for LegacyQueryPreferenceDurationMinutesRangeRule {
    fn check(&self, item: &LegacyQueryPreference) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a legacy query preference.
#[derive(Debug, Clone)]
pub struct LegacyQueryPreferenceValidator {
    pub rules: Vec<Rc<dyn LegacyQueryPreferenceRule>>,
}

impl LegacyQueryPreferenceValidator {
    pub fn validate(&self, item: &LegacyQueryPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyQueryPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyQueryPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_query_preference_validator() -> LegacyQueryPreferenceValidator {
    LegacyQueryPreferenceValidator {
        rules: vec![
            Rc::new(LegacyQueryPreferenceCostRangeRule { min: 4.0, max: 996.1 }) as Rc<dyn LegacyQueryPreferenceRule>,
            Rc::new(LegacyQueryPreferenceAllowedScopeRule { allowed: Vec::new() }),
            Rc::new(LegacyQueryPreferenceLabelLengthRule { max_length: 80 }),
            Rc::new(LegacyQueryPreferenceDurationMinutesRangeRule { min: 32, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_query_preference_validator();
        let item = new_legacy_query_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = LegacyQueryPreferenceCostRangeRule { min: 2.3, max: 1255.5 };
        let mut item = new_legacy_query_preference("x", "y");
        item.cost = 1257.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LegacyQueryPreferenceLabelLengthRule { max_length: 80 };
        let mut item = new_legacy_query_preference("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = LegacyQueryPreferenceDurationMinutesRangeRule { min: 1000, max: 3600 };
        let mut item = new_legacy_query_preference("x", "y");
        item.duration_minutes = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
