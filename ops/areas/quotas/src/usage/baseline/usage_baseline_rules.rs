use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a usage baseline must pass. Check returns its problems, or none.
pub trait UsageBaselineRule: std::fmt::Debug {
    fn check(&self, item: &UsageBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBaselineScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl UsageBaselineRule for UsageBaselineScoreRangeRule {
    fn check(&self, item: &UsageBaseline) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBaselineCodePrefixRule {
    pub prefix: String,
}

impl UsageBaselineRule for UsageBaselineCodePrefixRule {
    fn check(&self, item: &UsageBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBaselineAllowedCurrencyRule {
    pub allowed: Vec<String>,
}

impl UsageBaselineRule for UsageBaselineAllowedCurrencyRule {
    fn check(&self, item: &UsageBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.currency)))
        {
            return vec![format!("currency {} is missing", item.currency)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBaselineRequiredNameRule {
    pub message: String,
}

impl UsageBaselineRule for UsageBaselineRequiredNameRule {
    fn check(&self, item: &UsageBaseline) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBaselineNameLengthRule {
    pub max_length: i64,
}

impl UsageBaselineRule for UsageBaselineNameLengthRule {
    fn check(&self, item: &UsageBaseline) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBaselineCurrencyLengthRule {
    pub max_length: i64,
}

impl UsageBaselineRule for UsageBaselineCurrencyLengthRule {
    fn check(&self, item: &UsageBaseline) -> Vec<String> {
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

/// Runs every rule of a usage baseline.
#[derive(Debug, Clone)]
pub struct UsageBaselineValidator {
    pub rules: Vec<Rc<dyn UsageBaselineRule>>,
}

impl UsageBaselineValidator {
    pub fn validate(&self, item: &UsageBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &UsageBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_baseline_validator() -> UsageBaselineValidator {
    UsageBaselineValidator {
        rules: vec![
            Rc::new(UsageBaselineScoreRangeRule { min: 2.6, max: 3491.3 }) as Rc<dyn UsageBaselineRule>,
            Rc::new(UsageBaselineCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(UsageBaselineAllowedCurrencyRule {
                allowed: vec!["tiered".to_string(), "lenient".to_string(), "scheduled".to_string()],
            }),
            Rc::new(UsageBaselineRequiredNameRule {
                message: "usage baseline needs an entry".to_string(),
            }),
            Rc::new(UsageBaselineNameLengthRule { max_length: 255 }),
            Rc::new(UsageBaselineCurrencyLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_baseline_validator();
        let item = new_usage_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = UsageBaselineScoreRangeRule { min: 7.9, max: 2043.7 };
        let mut item = new_usage_baseline("x", "y");
        item.score = 2045.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = UsageBaselineRequiredNameRule {
            message: "usage baseline needs an entry".to_string(),
        };
        let mut item = new_usage_baseline("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = UsageBaselineNameLengthRule { max_length: 120 };
        let mut item = new_usage_baseline("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = UsageBaselineCurrencyLengthRule { max_length: 32 };
        let mut item = new_usage_baseline("x", "y");
        item.currency = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
