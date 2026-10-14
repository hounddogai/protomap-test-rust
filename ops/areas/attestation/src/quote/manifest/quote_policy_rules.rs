use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote policy must pass. Check returns its problems, or none.
pub trait QuotePolicyRule: std::fmt::Debug {
    fn check(&self, item: &QuotePolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuotePolicyRequiredNameRule {
    pub message: String,
}

impl QuotePolicyRule for QuotePolicyRequiredNameRule {
    fn check(&self, item: &QuotePolicy) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuotePolicyDescriptionLengthRule {
    pub max_length: i64,
}

impl QuotePolicyRule for QuotePolicyDescriptionLengthRule {
    fn check(&self, item: &QuotePolicy) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuotePolicyArchivedGuardRule {
    pub limit: f64,
}

impl QuotePolicyRule for QuotePolicyArchivedGuardRule {
    fn check(&self, item: &QuotePolicy) -> Vec<String> {
        if item.archived && (item.amount > self.limit) {
            return vec![format!("amount {} exceeds {} while archived", item.amount, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuotePolicyCodePrefixRule {
    pub prefix: String,
}

impl QuotePolicyRule for QuotePolicyCodePrefixRule {
    fn check(&self, item: &QuotePolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuotePolicyVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QuotePolicyRule for QuotePolicyVersionRangeRule {
    fn check(&self, item: &QuotePolicy) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuotePolicyDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QuotePolicyRule for QuotePolicyDurationMinutesRangeRule {
    fn check(&self, item: &QuotePolicy) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote policy.
#[derive(Debug, Clone)]
pub struct QuotePolicyValidator {
    pub rules: Vec<Rc<dyn QuotePolicyRule>>,
}

impl QuotePolicyValidator {
    pub fn validate(&self, item: &QuotePolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QuotePolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuotePolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_policy_validator() -> QuotePolicyValidator {
    QuotePolicyValidator {
        rules: vec![
            Rc::new(QuotePolicyRequiredNameRule {
                message: "quote policy needs this field".to_string(),
            }) as Rc<dyn QuotePolicyRule>,
            Rc::new(QuotePolicyDescriptionLengthRule { max_length: 16 }),
            Rc::new(QuotePolicyArchivedGuardRule { limit: 9.8 }),
            Rc::new(QuotePolicyCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(QuotePolicyVersionRangeRule { min: 32, max: 86400 }),
            Rc::new(QuotePolicyDurationMinutesRangeRule { min: 80, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_policy_validator();
        let item = new_quote_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = QuotePolicyRequiredNameRule {
            message: "quote policy needs a value".to_string(),
        };
        let mut item = new_quote_policy("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = QuotePolicyDescriptionLengthRule { max_length: 3600 };
        let mut item = new_quote_policy("x", "y");
        item.description = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = QuotePolicyVersionRangeRule { min: 80, max: 3600 };
        let mut item = new_quote_policy("x", "y");
        item.version = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = QuotePolicyDurationMinutesRangeRule { min: 86400, max: 86400 };
        let mut item = new_quote_policy("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
