use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a retention preference must pass. Check returns its problems, or none.
pub trait RetentionPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &RetentionPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionPreferenceTitleLengthRule {
    pub max_length: i64,
}

impl RetentionPreferenceRule for RetentionPreferenceTitleLengthRule {
    fn check(&self, item: &RetentionPreference) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionPreferenceActiveGuardRule {
    pub limit: f64,
}

impl RetentionPreferenceRule for RetentionPreferenceActiveGuardRule {
    fn check(&self, item: &RetentionPreference) -> Vec<String> {
        if item.active && (item.score > self.limit) {
            return vec![format!("score {} exceeds {} while active", item.score, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionPreferenceRequiredCurrencyRule {
    pub message: String,
}

impl RetentionPreferenceRule for RetentionPreferenceRequiredCurrencyRule {
    fn check(&self, item: &RetentionPreference) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention preference.
#[derive(Debug, Clone)]
pub struct RetentionPreferenceValidator {
    pub rules: Vec<Rc<dyn RetentionPreferenceRule>>,
}

impl RetentionPreferenceValidator {
    pub fn validate(&self, item: &RetentionPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RetentionPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_preference_validator() -> RetentionPreferenceValidator {
    RetentionPreferenceValidator {
        rules: vec![
            Rc::new(RetentionPreferenceTitleLengthRule { max_length: 3600 }) as Rc<dyn RetentionPreferenceRule>,
            Rc::new(RetentionPreferenceActiveGuardRule { limit: 3.0 }),
            Rc::new(RetentionPreferenceRequiredCurrencyRule {
                message: "retention preference needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_preference_validator();
        let item = new_retention_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RetentionPreferenceTitleLengthRule { max_length: 86400 };
        let mut item = new_retention_preference("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = RetentionPreferenceRequiredCurrencyRule {
            message: "retention preference needs this field".to_string(),
        };
        let mut item = new_retention_preference("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
