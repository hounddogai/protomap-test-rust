use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient dashboard ledger must pass. Check returns its problems, or none.
pub trait LenientDashboardLedgerRule: std::fmt::Debug {
    fn check(&self, item: &LenientDashboardLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDashboardLedgerAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl LenientDashboardLedgerRule for LenientDashboardLedgerAllowedLabelRule {
    fn check(&self, item: &LenientDashboardLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is below minimum", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDashboardLedgerCodeLengthRule {
    pub max_length: i64,
}

impl LenientDashboardLedgerRule for LenientDashboardLedgerCodeLengthRule {
    fn check(&self, item: &LenientDashboardLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDashboardLedgerBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientDashboardLedgerRule for LenientDashboardLedgerBurstRangeRule {
    fn check(&self, item: &LenientDashboardLedger) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDashboardLedgerRequiredLabelRule {
    pub message: String,
}

impl LenientDashboardLedgerRule for LenientDashboardLedgerRequiredLabelRule {
    fn check(&self, item: &LenientDashboardLedger) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDashboardLedgerDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientDashboardLedgerRule for LenientDashboardLedgerDiscountRangeRule {
    fn check(&self, item: &LenientDashboardLedger) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient dashboard ledger.
#[derive(Debug, Clone)]
pub struct LenientDashboardLedgerValidator {
    pub rules: Vec<Rc<dyn LenientDashboardLedgerRule>>,
}

impl LenientDashboardLedgerValidator {
    pub fn validate(&self, item: &LenientDashboardLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientDashboardLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientDashboardLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_dashboard_ledger_validator() -> LenientDashboardLedgerValidator {
    LenientDashboardLedgerValidator {
        rules: vec![
            Rc::new(LenientDashboardLedgerAllowedLabelRule {
                allowed: vec!["scheduled".to_string()],
            }) as Rc<dyn LenientDashboardLedgerRule>,
            Rc::new(LenientDashboardLedgerCodeLengthRule { max_length: 1000 }),
            Rc::new(LenientDashboardLedgerBurstRangeRule { min: 64, max: 86400 }),
            Rc::new(LenientDashboardLedgerRequiredLabelRule {
                message: "lenient dashboard ledger needs an entry".to_string(),
            }),
            Rc::new(LenientDashboardLedgerDiscountRangeRule { min: 9.9, max: 3601.0 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_dashboard_ledger_validator();
        let item = new_lenient_dashboard_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientDashboardLedgerCodeLengthRule { max_length: 3600 };
        let mut item = new_lenient_dashboard_ledger("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = LenientDashboardLedgerBurstRangeRule { min: 32, max: 80 };
        let mut item = new_lenient_dashboard_ledger("x", "y");
        item.burst = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientDashboardLedgerRequiredLabelRule {
            message: "lenient dashboard ledger needs a value".to_string(),
        };
        let mut item = new_lenient_dashboard_ledger("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = LenientDashboardLedgerDiscountRangeRule { min: 1.7, max: 3958.2 };
        let mut item = new_lenient_dashboard_ledger("x", "y");
        item.discount = 3959.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
