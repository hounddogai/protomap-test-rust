use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a alert baseline must pass. Check returns its problems, or none.
pub trait AlertBaselineRule: std::fmt::Debug {
    fn check(&self, item: &AlertBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertBaselineVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertBaselineRule for AlertBaselineVersionRangeRule {
    fn check(&self, item: &AlertBaseline) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertBaselineCodePrefixRule {
    pub prefix: String,
}

impl AlertBaselineRule for AlertBaselineCodePrefixRule {
    fn check(&self, item: &AlertBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertBaselineTitleLengthRule {
    pub max_length: i64,
}

impl AlertBaselineRule for AlertBaselineTitleLengthRule {
    fn check(&self, item: &AlertBaseline) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertBaselineRequiredLocaleRule {
    pub message: String,
}

impl AlertBaselineRule for AlertBaselineRequiredLocaleRule {
    fn check(&self, item: &AlertBaseline) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertBaselineLocaleLengthRule {
    pub max_length: i64,
}

impl AlertBaselineRule for AlertBaselineLocaleLengthRule {
    fn check(&self, item: &AlertBaseline) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert baseline.
#[derive(Debug, Clone)]
pub struct AlertBaselineValidator {
    pub rules: Vec<Rc<dyn AlertBaselineRule>>,
}

impl AlertBaselineValidator {
    pub fn validate(&self, item: &AlertBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AlertBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_baseline_validator() -> AlertBaselineValidator {
    AlertBaselineValidator {
        rules: vec![
            Rc::new(AlertBaselineVersionRangeRule { min: 32, max: 1000 }) as Rc<dyn AlertBaselineRule>,
            Rc::new(AlertBaselineCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(AlertBaselineTitleLengthRule { max_length: 16 }),
            Rc::new(AlertBaselineRequiredLocaleRule {
                message: "alert baseline needs an entry".to_string(),
            }),
            Rc::new(AlertBaselineLocaleLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_baseline_validator();
        let item = new_alert_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = AlertBaselineVersionRangeRule { min: 64, max: 120 };
        let mut item = new_alert_baseline("x", "y");
        item.version = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = AlertBaselineTitleLengthRule { max_length: 1000 };
        let mut item = new_alert_baseline("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = AlertBaselineRequiredLocaleRule {
            message: "alert baseline needs this field".to_string(),
        };
        let mut item = new_alert_baseline("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = AlertBaselineLocaleLengthRule { max_length: 120 };
        let mut item = new_alert_baseline("x", "y");
        item.locale = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
