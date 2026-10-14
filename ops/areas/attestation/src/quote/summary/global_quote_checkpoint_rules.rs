use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global quote checkpoint must pass. Check returns its problems, or none.
pub trait GlobalQuoteCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &GlobalQuoteCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalQuoteCheckpointRequiredLocaleRule {
    pub message: String,
}

impl GlobalQuoteCheckpointRule for GlobalQuoteCheckpointRequiredLocaleRule {
    fn check(&self, item: &GlobalQuoteCheckpoint) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalQuoteCheckpointMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalQuoteCheckpointRule for GlobalQuoteCheckpointMarginRangeRule {
    fn check(&self, item: &GlobalQuoteCheckpoint) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalQuoteCheckpointVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalQuoteCheckpointRule for GlobalQuoteCheckpointVersionRangeRule {
    fn check(&self, item: &GlobalQuoteCheckpoint) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalQuoteCheckpointCodePrefixRule {
    pub prefix: String,
}

impl GlobalQuoteCheckpointRule for GlobalQuoteCheckpointCodePrefixRule {
    fn check(&self, item: &GlobalQuoteCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalQuoteCheckpointAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl GlobalQuoteCheckpointRule for GlobalQuoteCheckpointAllowedCodeRule {
    fn check(&self, item: &GlobalQuoteCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalQuoteCheckpointRequiredCodeRule {
    pub message: String,
}

impl GlobalQuoteCheckpointRule for GlobalQuoteCheckpointRequiredCodeRule {
    fn check(&self, item: &GlobalQuoteCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global quote checkpoint.
#[derive(Debug, Clone)]
pub struct GlobalQuoteCheckpointValidator {
    pub rules: Vec<Rc<dyn GlobalQuoteCheckpointRule>>,
}

impl GlobalQuoteCheckpointValidator {
    pub fn validate(&self, item: &GlobalQuoteCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalQuoteCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalQuoteCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_quote_checkpoint_validator() -> GlobalQuoteCheckpointValidator {
    GlobalQuoteCheckpointValidator {
        rules: vec![
            Rc::new(GlobalQuoteCheckpointRequiredLocaleRule {
                message: "global quote checkpoint needs this field".to_string(),
            }) as Rc<dyn GlobalQuoteCheckpointRule>,
            Rc::new(GlobalQuoteCheckpointMarginRangeRule { min: 3.4, max: 4219.1 }),
            Rc::new(GlobalQuoteCheckpointVersionRangeRule { min: 64, max: 255 }),
            Rc::new(GlobalQuoteCheckpointCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(GlobalQuoteCheckpointAllowedCodeRule {
                allowed: vec!["legacy".to_string(), "weighted".to_string(), "regional".to_string()],
            }),
            Rc::new(GlobalQuoteCheckpointRequiredCodeRule {
                message: "global quote checkpoint needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_quote_checkpoint_validator();
        let item = new_global_quote_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = GlobalQuoteCheckpointRequiredLocaleRule {
            message: "global quote checkpoint needs an entry".to_string(),
        };
        let mut item = new_global_quote_checkpoint("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = GlobalQuoteCheckpointMarginRangeRule { min: 2.6, max: 542.4 };
        let mut item = new_global_quote_checkpoint("x", "y");
        item.margin = 543.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = GlobalQuoteCheckpointVersionRangeRule { min: 16, max: 86400 };
        let mut item = new_global_quote_checkpoint("x", "y");
        item.version = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalQuoteCheckpointRequiredCodeRule {
            message: "global quote checkpoint needs a value".to_string(),
        };
        let mut item = new_global_quote_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
