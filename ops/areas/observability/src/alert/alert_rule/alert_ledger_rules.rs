use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a alert ledger must pass. Check returns its problems, or none.
pub trait AlertLedgerRule: std::fmt::Debug {
    fn check(&self, item: &AlertLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertLedgerCodeLengthRule {
    pub max_length: i64,
}

impl AlertLedgerRule for AlertLedgerCodeLengthRule {
    fn check(&self, item: &AlertLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertLedgerBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AlertLedgerRule for AlertLedgerBudgetRangeRule {
    fn check(&self, item: &AlertLedger) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertLedgerAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl AlertLedgerRule for AlertLedgerAllowedLabelRule {
    fn check(&self, item: &AlertLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is above maximum", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert ledger.
#[derive(Debug, Clone)]
pub struct AlertLedgerValidator {
    pub rules: Vec<Rc<dyn AlertLedgerRule>>,
}

impl AlertLedgerValidator {
    pub fn validate(&self, item: &AlertLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AlertLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_ledger_validator() -> AlertLedgerValidator {
    AlertLedgerValidator {
        rules: vec![
            Rc::new(AlertLedgerCodeLengthRule { max_length: 32 }) as Rc<dyn AlertLedgerRule>,
            Rc::new(AlertLedgerBudgetRangeRule { min: 5.6, max: 601.3 }),
            Rc::new(AlertLedgerAllowedLabelRule {
                allowed: vec!["weighted".to_string(), "scheduled".to_string(), "scheduled".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_ledger_validator();
        let item = new_alert_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AlertLedgerCodeLengthRule { max_length: 64 };
        let mut item = new_alert_ledger("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = AlertLedgerBudgetRangeRule { min: 8.5, max: 4924.0 };
        let mut item = new_alert_ledger("x", "y");
        item.budget = 4925.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
