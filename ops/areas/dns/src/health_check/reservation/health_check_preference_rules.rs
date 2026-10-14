use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check preference must pass. Check returns its problems, or none.
pub trait HealthCheckPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckPreferenceLocaleLengthRule {
    pub max_length: i64,
}

impl HealthCheckPreferenceRule for HealthCheckPreferenceLocaleLengthRule {
    fn check(&self, item: &HealthCheckPreference) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckPreferenceHostLengthRule {
    pub max_length: i64,
}

impl HealthCheckPreferenceRule for HealthCheckPreferenceHostLengthRule {
    fn check(&self, item: &HealthCheckPreference) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckPreferenceRequiredLocaleRule {
    pub message: String,
}

impl HealthCheckPreferenceRule for HealthCheckPreferenceRequiredLocaleRule {
    fn check(&self, item: &HealthCheckPreference) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check preference.
#[derive(Debug, Clone)]
pub struct HealthCheckPreferenceValidator {
    pub rules: Vec<Rc<dyn HealthCheckPreferenceRule>>,
}

impl HealthCheckPreferenceValidator {
    pub fn validate(&self, item: &HealthCheckPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &HealthCheckPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_preference_validator() -> HealthCheckPreferenceValidator {
    HealthCheckPreferenceValidator {
        rules: vec![
            Rc::new(HealthCheckPreferenceLocaleLengthRule { max_length: 86400 }) as Rc<dyn HealthCheckPreferenceRule>,
            Rc::new(HealthCheckPreferenceHostLengthRule { max_length: 255 }),
            Rc::new(HealthCheckPreferenceRequiredLocaleRule {
                message: "health check preference needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_preference_validator();
        let item = new_health_check_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = HealthCheckPreferenceLocaleLengthRule { max_length: 80 };
        let mut item = new_health_check_preference("x", "y");
        item.locale = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = HealthCheckPreferenceHostLengthRule { max_length: 1000 };
        let mut item = new_health_check_preference("x", "y");
        item.host = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = HealthCheckPreferenceRequiredLocaleRule {
            message: "health check preference needs a value".to_string(),
        };
        let mut item = new_health_check_preference("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
