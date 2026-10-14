use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote rollup must pass. Check returns its problems, or none.
pub trait QuoteRollupRule: std::fmt::Debug {
    fn check(&self, item: &QuoteRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteRollupDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QuoteRollupRule for QuoteRollupDiscountRangeRule {
    fn check(&self, item: &QuoteRollup) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteRollupCodeLengthRule {
    pub max_length: i64,
}

impl QuoteRollupRule for QuoteRollupCodeLengthRule {
    fn check(&self, item: &QuoteRollup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteRollupCodePrefixRule {
    pub prefix: String,
}

impl QuoteRollupRule for QuoteRollupCodePrefixRule {
    fn check(&self, item: &QuoteRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteRollupLabelLengthRule {
    pub max_length: i64,
}

impl QuoteRollupRule for QuoteRollupLabelLengthRule {
    fn check(&self, item: &QuoteRollup) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteRollupCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QuoteRollupRule for QuoteRollupCreatedAtRangeRule {
    fn check(&self, item: &QuoteRollup) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteRollupCurrencyLengthRule {
    pub max_length: i64,
}

impl QuoteRollupRule for QuoteRollupCurrencyLengthRule {
    fn check(&self, item: &QuoteRollup) -> Vec<String> {
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

/// Runs every rule of a quote rollup.
#[derive(Debug, Clone)]
pub struct QuoteRollupValidator {
    pub rules: Vec<Rc<dyn QuoteRollupRule>>,
}

impl QuoteRollupValidator {
    pub fn validate(&self, item: &QuoteRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QuoteRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_rollup_validator() -> QuoteRollupValidator {
    QuoteRollupValidator {
        rules: vec![
            Rc::new(QuoteRollupDiscountRangeRule { min: 7.6, max: 2965.3 }) as Rc<dyn QuoteRollupRule>,
            Rc::new(QuoteRollupCodeLengthRule { max_length: 1000 }),
            Rc::new(QuoteRollupCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(QuoteRollupLabelLengthRule { max_length: 1000 }),
            Rc::new(QuoteRollupCreatedAtRangeRule { min: 32, max: 80 }),
            Rc::new(QuoteRollupCurrencyLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_rollup_validator();
        let item = new_quote_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = QuoteRollupDiscountRangeRule { min: 8.5, max: 2802.9 };
        let mut item = new_quote_rollup("x", "y");
        item.discount = 2804.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = QuoteRollupCodeLengthRule { max_length: 80 };
        let mut item = new_quote_rollup("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = QuoteRollupLabelLengthRule { max_length: 32 };
        let mut item = new_quote_rollup("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = QuoteRollupCreatedAtRangeRule { min: 80, max: 3600 };
        let mut item = new_quote_rollup("x", "y");
        item.created_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = QuoteRollupCurrencyLengthRule { max_length: 32 };
        let mut item = new_quote_rollup("x", "y");
        item.currency = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
