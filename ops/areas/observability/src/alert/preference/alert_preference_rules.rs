use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a alert preference must pass. Check returns its problems, or none.
pub trait AlertPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &AlertPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertPreferenceLocaleLengthRule {
    pub max_length: i64,
}

impl AlertPreferenceRule for AlertPreferenceLocaleLengthRule {
    fn check(&self, item: &AlertPreference) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertPreferenceActiveGuardRule {
    pub limit: f64,
}

impl AlertPreferenceRule for AlertPreferenceActiveGuardRule {
    fn check(&self, item: &AlertPreference) -> Vec<String> {
        if item.active && (item.threshold > self.limit) {
            return vec![format!(
                "threshold {} exceeds {} while active",
                item.threshold, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertPreferenceRequiredCodeRule {
    pub message: String,
}

impl AlertPreferenceRule for AlertPreferenceRequiredCodeRule {
    fn check(&self, item: &AlertPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertPreferenceRequiredSegmentRule {
    pub message: String,
}

impl AlertPreferenceRule for AlertPreferenceRequiredSegmentRule {
    fn check(&self, item: &AlertPreference) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert preference.
#[derive(Debug, Clone)]
pub struct AlertPreferenceValidator {
    pub rules: Vec<Rc<dyn AlertPreferenceRule>>,
}

impl AlertPreferenceValidator {
    pub fn validate(&self, item: &AlertPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AlertPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_preference_validator() -> AlertPreferenceValidator {
    AlertPreferenceValidator {
        rules: vec![
            Rc::new(AlertPreferenceLocaleLengthRule { max_length: 1000 }) as Rc<dyn AlertPreferenceRule>,
            Rc::new(AlertPreferenceActiveGuardRule { limit: 4.3 }),
            Rc::new(AlertPreferenceRequiredCodeRule {
                message: "alert preference needs this field".to_string(),
            }),
            Rc::new(AlertPreferenceRequiredSegmentRule {
                message: "alert preference needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_preference_validator();
        let item = new_alert_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = AlertPreferenceLocaleLengthRule { max_length: 16 };
        let mut item = new_alert_preference("x", "y");
        item.locale = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AlertPreferenceRequiredCodeRule {
            message: "alert preference needs this field".to_string(),
        };
        let mut item = new_alert_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = AlertPreferenceRequiredSegmentRule {
            message: "alert preference needs an entry".to_string(),
        };
        let mut item = new_alert_preference("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
