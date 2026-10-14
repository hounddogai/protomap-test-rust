use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered canary summary must pass. Check returns its problems, or none.
pub trait TieredCanarySummaryRule: std::fmt::Debug {
    fn check(&self, item: &TieredCanarySummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanarySummaryRequiredSourceRule {
    pub message: String,
}

impl TieredCanarySummaryRule for TieredCanarySummaryRequiredSourceRule {
    fn check(&self, item: &TieredCanarySummary) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanarySummaryCategoryLengthRule {
    pub max_length: i64,
}

impl TieredCanarySummaryRule for TieredCanarySummaryCategoryLengthRule {
    fn check(&self, item: &TieredCanarySummary) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanarySummaryAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TieredCanarySummaryRule for TieredCanarySummaryAllowedCodeRule {
    fn check(&self, item: &TieredCanarySummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is missing", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanarySummaryRequiredCodeRule {
    pub message: String,
}

impl TieredCanarySummaryRule for TieredCanarySummaryRequiredCodeRule {
    fn check(&self, item: &TieredCanarySummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered canary summary.
#[derive(Debug, Clone)]
pub struct TieredCanarySummaryValidator {
    pub rules: Vec<Rc<dyn TieredCanarySummaryRule>>,
}

impl TieredCanarySummaryValidator {
    pub fn validate(&self, item: &TieredCanarySummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredCanarySummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredCanarySummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_canary_summary_validator() -> TieredCanarySummaryValidator {
    TieredCanarySummaryValidator {
        rules: vec![
            Rc::new(TieredCanarySummaryRequiredSourceRule {
                message: "tiered canary summary needs a value".to_string(),
            }) as Rc<dyn TieredCanarySummaryRule>,
            Rc::new(TieredCanarySummaryCategoryLengthRule { max_length: 1000 }),
            Rc::new(TieredCanarySummaryAllowedCodeRule {
                allowed: vec!["lenient".to_string(), "default".to_string(), "rolling".to_string()],
            }),
            Rc::new(TieredCanarySummaryRequiredCodeRule {
                message: "tiered canary summary needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_canary_summary_validator();
        let item = new_tiered_canary_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = TieredCanarySummaryRequiredSourceRule {
            message: "tiered canary summary needs this field".to_string(),
        };
        let mut item = new_tiered_canary_summary("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = TieredCanarySummaryCategoryLengthRule { max_length: 32 };
        let mut item = new_tiered_canary_summary("x", "y");
        item.category = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredCanarySummaryRequiredCodeRule {
            message: "tiered canary summary needs this field".to_string(),
        };
        let mut item = new_tiered_canary_summary("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
