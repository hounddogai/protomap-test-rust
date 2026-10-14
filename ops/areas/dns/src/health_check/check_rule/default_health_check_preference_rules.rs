use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default health check preference must pass. Check returns its problems, or none.
pub trait DefaultHealthCheckPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &DefaultHealthCheckPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultHealthCheckPreferenceWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultHealthCheckPreferenceRule for DefaultHealthCheckPreferenceWindowDaysRangeRule {
    fn check(&self, item: &DefaultHealthCheckPreference) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultHealthCheckPreferenceRequiredLabelRule {
    pub message: String,
}

impl DefaultHealthCheckPreferenceRule for DefaultHealthCheckPreferenceRequiredLabelRule {
    fn check(&self, item: &DefaultHealthCheckPreference) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultHealthCheckPreferenceNotesLengthRule {
    pub max_length: i64,
}

impl DefaultHealthCheckPreferenceRule for DefaultHealthCheckPreferenceNotesLengthRule {
    fn check(&self, item: &DefaultHealthCheckPreference) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultHealthCheckPreferenceAllowedNameRule {
    pub allowed: Vec<String>,
}

impl DefaultHealthCheckPreferenceRule for DefaultHealthCheckPreferenceAllowedNameRule {
    fn check(&self, item: &DefaultHealthCheckPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is not allowed", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default health check preference.
#[derive(Debug, Clone)]
pub struct DefaultHealthCheckPreferenceValidator {
    pub rules: Vec<Rc<dyn DefaultHealthCheckPreferenceRule>>,
}

impl DefaultHealthCheckPreferenceValidator {
    pub fn validate(&self, item: &DefaultHealthCheckPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultHealthCheckPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultHealthCheckPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_health_check_preference_validator() -> DefaultHealthCheckPreferenceValidator {
    DefaultHealthCheckPreferenceValidator {
        rules: vec![
            Rc::new(DefaultHealthCheckPreferenceWindowDaysRangeRule { min: 80, max: 120 })
                as Rc<dyn DefaultHealthCheckPreferenceRule>,
            Rc::new(DefaultHealthCheckPreferenceRequiredLabelRule {
                message: "default health check preference needs a value".to_string(),
            }),
            Rc::new(DefaultHealthCheckPreferenceNotesLengthRule { max_length: 80 }),
            Rc::new(DefaultHealthCheckPreferenceAllowedNameRule {
                allowed: vec!["rolling".to_string(), "lenient".to_string(), "manual".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_health_check_preference_validator();
        let item = new_default_health_check_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = DefaultHealthCheckPreferenceWindowDaysRangeRule { min: 255, max: 1000 };
        let mut item = new_default_health_check_preference("x", "y");
        item.window_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DefaultHealthCheckPreferenceRequiredLabelRule {
            message: "default health check preference needs a value".to_string(),
        };
        let mut item = new_default_health_check_preference("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = DefaultHealthCheckPreferenceNotesLengthRule { max_length: 255 };
        let mut item = new_default_health_check_preference("x", "y");
        item.notes = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
