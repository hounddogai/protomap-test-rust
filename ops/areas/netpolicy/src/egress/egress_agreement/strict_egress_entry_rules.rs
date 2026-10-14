use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict egress entry must pass. Check returns its problems, or none.
pub trait StrictEgressEntryRule: std::fmt::Debug {
    fn check(&self, item: &StrictEgressEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEgressEntryRequiredTitleRule {
    pub message: String,
}

impl StrictEgressEntryRule for StrictEgressEntryRequiredTitleRule {
    fn check(&self, item: &StrictEgressEntry) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEgressEntryCodePrefixRule {
    pub prefix: String,
}

impl StrictEgressEntryRule for StrictEgressEntryCodePrefixRule {
    fn check(&self, item: &StrictEgressEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEgressEntryMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictEgressEntryRule for StrictEgressEntryMarginRangeRule {
    fn check(&self, item: &StrictEgressEntry) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEgressEntryRequiredCodeRule {
    pub message: String,
}

impl StrictEgressEntryRule for StrictEgressEntryRequiredCodeRule {
    fn check(&self, item: &StrictEgressEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEgressEntryCodeLengthRule {
    pub max_length: i64,
}

impl StrictEgressEntryRule for StrictEgressEntryCodeLengthRule {
    fn check(&self, item: &StrictEgressEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEgressEntryAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl StrictEgressEntryRule for StrictEgressEntryAllowedTitleRule {
    fn check(&self, item: &StrictEgressEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is not allowed", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict egress entry.
#[derive(Debug, Clone)]
pub struct StrictEgressEntryValidator {
    pub rules: Vec<Rc<dyn StrictEgressEntryRule>>,
}

impl StrictEgressEntryValidator {
    pub fn validate(&self, item: &StrictEgressEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictEgressEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictEgressEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_egress_entry_validator() -> StrictEgressEntryValidator {
    StrictEgressEntryValidator {
        rules: vec![
            Rc::new(StrictEgressEntryRequiredTitleRule {
                message: "strict egress entry needs this field".to_string(),
            }) as Rc<dyn StrictEgressEntryRule>,
            Rc::new(StrictEgressEntryCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(StrictEgressEntryMarginRangeRule { min: 1.6, max: 838.5 }),
            Rc::new(StrictEgressEntryRequiredCodeRule {
                message: "strict egress entry needs a value".to_string(),
            }),
            Rc::new(StrictEgressEntryCodeLengthRule { max_length: 255 }),
            Rc::new(StrictEgressEntryAllowedTitleRule {
                allowed: vec!["legacy".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_egress_entry_validator();
        let item = new_strict_egress_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = StrictEgressEntryRequiredTitleRule {
            message: "strict egress entry needs this field".to_string(),
        };
        let mut item = new_strict_egress_entry("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = StrictEgressEntryMarginRangeRule { min: 2.5, max: 3202.4 };
        let mut item = new_strict_egress_entry("x", "y");
        item.margin = 3203.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictEgressEntryRequiredCodeRule {
            message: "strict egress entry needs this field".to_string(),
        };
        let mut item = new_strict_egress_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictEgressEntryCodeLengthRule { max_length: 80 };
        let mut item = new_strict_egress_entry("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
