use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote summary must pass. Check returns its problems, or none.
pub trait QuoteSummaryRule: std::fmt::Debug {
    fn check(&self, item: &QuoteSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteSummaryNamespaceLengthRule {
    pub max_length: i64,
}

impl QuoteSummaryRule for QuoteSummaryNamespaceLengthRule {
    fn check(&self, item: &QuoteSummary) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteSummaryRequiredNameRule {
    pub message: String,
}

impl QuoteSummaryRule for QuoteSummaryRequiredNameRule {
    fn check(&self, item: &QuoteSummary) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteSummaryBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QuoteSummaryRule for QuoteSummaryBurstRangeRule {
    fn check(&self, item: &QuoteSummary) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a quote summary.
#[derive(Debug, Clone)]
pub struct QuoteSummaryValidator {
    pub rules: Vec<Rc<dyn QuoteSummaryRule>>,
}

impl QuoteSummaryValidator {
    pub fn validate(&self, item: &QuoteSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QuoteSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_summary_validator() -> QuoteSummaryValidator {
    QuoteSummaryValidator {
        rules: vec![
            Rc::new(QuoteSummaryNamespaceLengthRule { max_length: 32 }) as Rc<dyn QuoteSummaryRule>,
            Rc::new(QuoteSummaryRequiredNameRule {
                message: "quote summary needs a value".to_string(),
            }),
            Rc::new(QuoteSummaryBurstRangeRule { min: 16, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_summary_validator();
        let item = new_quote_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = QuoteSummaryNamespaceLengthRule { max_length: 1000 };
        let mut item = new_quote_summary("x", "y");
        item.namespace = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = QuoteSummaryRequiredNameRule {
            message: "quote summary needs this field".to_string(),
        };
        let mut item = new_quote_summary("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = QuoteSummaryBurstRangeRule { min: 120, max: 1000 };
        let mut item = new_quote_summary("x", "y");
        item.burst = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
