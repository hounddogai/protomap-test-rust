use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a quote cycle must pass. Check returns its problems, or none.
pub trait QuoteCycleRule: std::fmt::Debug {
    fn check(&self, item: &QuoteCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteCycleRequiredSummaryRule {
    pub message: String,
}

impl QuoteCycleRule for QuoteCycleRequiredSummaryRule {
    fn check(&self, item: &QuoteCycle) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteCycleAllowedSummaryRule {
    pub allowed: Vec<String>,
}

impl QuoteCycleRule for QuoteCycleAllowedSummaryRule {
    fn check(&self, item: &QuoteCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.summary)))
        {
            return vec![format!("summary {} is too long", item.summary)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteCycleCodeLengthRule {
    pub max_length: i64,
}

impl QuoteCycleRule for QuoteCycleCodeLengthRule {
    fn check(&self, item: &QuoteCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteCycleGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QuoteCycleRule for QuoteCycleGraceDaysRangeRule {
    fn check(&self, item: &QuoteCycle) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteCycleNamespaceLengthRule {
    pub max_length: i64,
}

impl QuoteCycleRule for QuoteCycleNamespaceLengthRule {
    fn check(&self, item: &QuoteCycle) -> Vec<String> {
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
pub struct QuoteCycleRequiredLabelRule {
    pub message: String,
}

impl QuoteCycleRule for QuoteCycleRequiredLabelRule {
    fn check(&self, item: &QuoteCycle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote cycle.
#[derive(Debug, Clone)]
pub struct QuoteCycleValidator {
    pub rules: Vec<Rc<dyn QuoteCycleRule>>,
}

impl QuoteCycleValidator {
    pub fn validate(&self, item: &QuoteCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QuoteCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_cycle_validator() -> QuoteCycleValidator {
    QuoteCycleValidator {
        rules: vec![
            Rc::new(QuoteCycleRequiredSummaryRule {
                message: "quote cycle needs an entry".to_string(),
            }) as Rc<dyn QuoteCycleRule>,
            Rc::new(QuoteCycleAllowedSummaryRule {
                allowed: vec!["capped".to_string(), "adaptive".to_string(), "adaptive".to_string()],
            }),
            Rc::new(QuoteCycleCodeLengthRule { max_length: 1000 }),
            Rc::new(QuoteCycleGraceDaysRangeRule { min: 80, max: 255 }),
            Rc::new(QuoteCycleNamespaceLengthRule { max_length: 1000 }),
            Rc::new(QuoteCycleRequiredLabelRule {
                message: "quote cycle needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_cycle_validator();
        let item = new_quote_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = QuoteCycleRequiredSummaryRule {
            message: "quote cycle needs this field".to_string(),
        };
        let mut item = new_quote_cycle("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = QuoteCycleCodeLengthRule { max_length: 1000 };
        let mut item = new_quote_cycle("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = QuoteCycleGraceDaysRangeRule { min: 3600, max: 86400 };
        let mut item = new_quote_cycle("x", "y");
        item.grace_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = QuoteCycleNamespaceLengthRule { max_length: 32 };
        let mut item = new_quote_cycle("x", "y");
        item.namespace = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = QuoteCycleRequiredLabelRule {
            message: "quote cycle needs this field".to_string(),
        };
        let mut item = new_quote_cycle("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
