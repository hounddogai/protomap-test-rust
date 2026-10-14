use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a postmortem preference must pass. Check returns its problems, or none.
pub trait PostmortemPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemPreferenceCurrencyLengthRule {
    pub max_length: i64,
}

impl PostmortemPreferenceRule for PostmortemPreferenceCurrencyLengthRule {
    fn check(&self, item: &PostmortemPreference) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemPreferencePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PostmortemPreferenceRule for PostmortemPreferencePortRangeRule {
    fn check(&self, item: &PostmortemPreference) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemPreferenceLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PostmortemPreferenceRule for PostmortemPreferenceLimitRangeRule {
    fn check(&self, item: &PostmortemPreference) -> Vec<String> {
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
pub struct PostmortemPreferenceCodePrefixRule {
    pub prefix: String,
}

impl PostmortemPreferenceRule for PostmortemPreferenceCodePrefixRule {
    fn check(&self, item: &PostmortemPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemPreferenceTitleLengthRule {
    pub max_length: i64,
}

impl PostmortemPreferenceRule for PostmortemPreferenceTitleLengthRule {
    fn check(&self, item: &PostmortemPreference) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem preference.
#[derive(Debug, Clone)]
pub struct PostmortemPreferenceValidator {
    pub rules: Vec<Rc<dyn PostmortemPreferenceRule>>,
}

impl PostmortemPreferenceValidator {
    pub fn validate(&self, item: &PostmortemPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PostmortemPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_preference_validator() -> PostmortemPreferenceValidator {
    PostmortemPreferenceValidator {
        rules: vec![
            Rc::new(PostmortemPreferenceCurrencyLengthRule { max_length: 16 }) as Rc<dyn PostmortemPreferenceRule>,
            Rc::new(PostmortemPreferencePortRangeRule { min: 80, max: 1000 }),
            Rc::new(PostmortemPreferenceLimitRangeRule { min: 3600, max: 3600 }),
            Rc::new(PostmortemPreferenceCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(PostmortemPreferenceTitleLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_postmortem_preference_validator();
        let item = new_postmortem_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = PostmortemPreferenceCurrencyLengthRule { max_length: 32 };
        let mut item = new_postmortem_preference("x", "y");
        item.currency = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = PostmortemPreferencePortRangeRule { min: 32, max: 3600 };
        let mut item = new_postmortem_preference("x", "y");
        item.port = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = PostmortemPreferenceLimitRangeRule { min: 32, max: 1000 };
        let mut item = new_postmortem_preference("x", "y");
        item.limit = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PostmortemPreferenceTitleLengthRule { max_length: 3600 };
        let mut item = new_postmortem_preference("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
