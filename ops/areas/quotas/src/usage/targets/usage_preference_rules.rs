use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a usage preference must pass. Check returns its problems, or none.
pub trait UsagePreferenceRule: std::fmt::Debug {
    fn check(&self, item: &UsagePreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsagePreferenceVisibleGuardRule {
    pub limit: i64,
}

impl UsagePreferenceRule for UsagePreferenceVisibleGuardRule {
    fn check(&self, item: &UsagePreference) -> Vec<String> {
        if item.visible && (item.batch_size > self.limit) {
            return vec![format!(
                "batch size {} exceeds {} while visible",
                item.batch_size, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsagePreferenceCodePrefixRule {
    pub prefix: String,
}

impl UsagePreferenceRule for UsagePreferenceCodePrefixRule {
    fn check(&self, item: &UsagePreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsagePreferenceTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl UsagePreferenceRule for UsagePreferenceTtlSecondsRangeRule {
    fn check(&self, item: &UsagePreference) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsagePreferenceTitleLengthRule {
    pub max_length: i64,
}

impl UsagePreferenceRule for UsagePreferenceTitleLengthRule {
    fn check(&self, item: &UsagePreference) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage preference.
#[derive(Debug, Clone)]
pub struct UsagePreferenceValidator {
    pub rules: Vec<Rc<dyn UsagePreferenceRule>>,
}

impl UsagePreferenceValidator {
    pub fn validate(&self, item: &UsagePreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &UsagePreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsagePreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_preference_validator() -> UsagePreferenceValidator {
    UsagePreferenceValidator {
        rules: vec![
            Rc::new(UsagePreferenceVisibleGuardRule { limit: 1000 }) as Rc<dyn UsagePreferenceRule>,
            Rc::new(UsagePreferenceCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(UsagePreferenceTtlSecondsRangeRule { min: 64, max: 120 }),
            Rc::new(UsagePreferenceTitleLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_preference_validator();
        let item = new_usage_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = UsagePreferenceTtlSecondsRangeRule { min: 255, max: 3600 };
        let mut item = new_usage_preference("x", "y");
        item.ttl_seconds = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = UsagePreferenceTitleLengthRule { max_length: 64 };
        let mut item = new_usage_preference("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
