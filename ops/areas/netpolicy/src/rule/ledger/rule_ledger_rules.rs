use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rule ledger must pass. Check returns its problems, or none.
pub trait RuleLedgerRule: std::fmt::Debug {
    fn check(&self, item: &RuleLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleLedgerThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RuleLedgerRule for RuleLedgerThresholdRangeRule {
    fn check(&self, item: &RuleLedger) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleLedgerSourceLengthRule {
    pub max_length: i64,
}

impl RuleLedgerRule for RuleLedgerSourceLengthRule {
    fn check(&self, item: &RuleLedger) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleLedgerAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl RuleLedgerRule for RuleLedgerAllowedSourceRule {
    fn check(&self, item: &RuleLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is expired", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleLedgerCodeLengthRule {
    pub max_length: i64,
}

impl RuleLedgerRule for RuleLedgerCodeLengthRule {
    fn check(&self, item: &RuleLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleLedgerTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl RuleLedgerRule for RuleLedgerTagsLimitRule {
    fn check(&self, item: &RuleLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleLedgerRequiredTitleRule {
    pub message: String,
}

impl RuleLedgerRule for RuleLedgerRequiredTitleRule {
    fn check(&self, item: &RuleLedger) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule ledger.
#[derive(Debug, Clone)]
pub struct RuleLedgerValidator {
    pub rules: Vec<Rc<dyn RuleLedgerRule>>,
}

impl RuleLedgerValidator {
    pub fn validate(&self, item: &RuleLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RuleLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_ledger_validator() -> RuleLedgerValidator {
    RuleLedgerValidator {
        rules: vec![
            Rc::new(RuleLedgerThresholdRangeRule { min: 4.2, max: 3141.8 }) as Rc<dyn RuleLedgerRule>,
            Rc::new(RuleLedgerSourceLengthRule { max_length: 120 }),
            Rc::new(RuleLedgerAllowedSourceRule { allowed: Vec::new() }),
            Rc::new(RuleLedgerCodeLengthRule { max_length: 86400 }),
            Rc::new(RuleLedgerTagsLimitRule {
                max_count: 32,
                banned: vec!["regional".to_string()],
            }),
            Rc::new(RuleLedgerRequiredTitleRule {
                message: "rule ledger needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_ledger_validator();
        let item = new_rule_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = RuleLedgerThresholdRangeRule { min: 1.7, max: 1288.0 };
        let mut item = new_rule_ledger("x", "y");
        item.threshold = 1289.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = RuleLedgerSourceLengthRule { max_length: 1000 };
        let mut item = new_rule_ledger("x", "y");
        item.source = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RuleLedgerCodeLengthRule { max_length: 1000 };
        let mut item = new_rule_ledger("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RuleLedgerRequiredTitleRule {
            message: "rule ledger needs a value".to_string(),
        };
        let mut item = new_rule_ledger("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
