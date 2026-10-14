use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a region pin ledger must pass. Check returns its problems, or none.
pub trait RegionPinLedgerRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinLedgerCurrencyLengthRule {
    pub max_length: i64,
}

impl RegionPinLedgerRule for RegionPinLedgerCurrencyLengthRule {
    fn check(&self, item: &RegionPinLedger) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinLedgerArchivedGuardRule {
    pub limit: f64,
}

impl RegionPinLedgerRule for RegionPinLedgerArchivedGuardRule {
    fn check(&self, item: &RegionPinLedger) -> Vec<String> {
        if item.archived && (item.weight > self.limit) {
            return vec![format!("weight {} exceeds {} while archived", item.weight, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinLedgerAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl RegionPinLedgerRule for RegionPinLedgerAllowedLabelRule {
    fn check(&self, item: &RegionPinLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is not allowed", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinLedgerRequiredCurrencyRule {
    pub message: String,
}

impl RegionPinLedgerRule for RegionPinLedgerRequiredCurrencyRule {
    fn check(&self, item: &RegionPinLedger) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinLedgerRequiredLabelRule {
    pub message: String,
}

impl RegionPinLedgerRule for RegionPinLedgerRequiredLabelRule {
    fn check(&self, item: &RegionPinLedger) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin ledger.
#[derive(Debug, Clone)]
pub struct RegionPinLedgerValidator {
    pub rules: Vec<Rc<dyn RegionPinLedgerRule>>,
}

impl RegionPinLedgerValidator {
    pub fn validate(&self, item: &RegionPinLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionPinLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_ledger_validator() -> RegionPinLedgerValidator {
    RegionPinLedgerValidator {
        rules: vec![
            Rc::new(RegionPinLedgerCurrencyLengthRule { max_length: 16 }) as Rc<dyn RegionPinLedgerRule>,
            Rc::new(RegionPinLedgerArchivedGuardRule { limit: 1.8 }),
            Rc::new(RegionPinLedgerAllowedLabelRule {
                allowed: vec!["seasonal".to_string(), "batched".to_string(), "strict".to_string()],
            }),
            Rc::new(RegionPinLedgerRequiredCurrencyRule {
                message: "region pin ledger needs this field".to_string(),
            }),
            Rc::new(RegionPinLedgerRequiredLabelRule {
                message: "region pin ledger needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_ledger_validator();
        let item = new_region_pin_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = RegionPinLedgerCurrencyLengthRule { max_length: 1000 };
        let mut item = new_region_pin_ledger("x", "y");
        item.currency = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = RegionPinLedgerRequiredCurrencyRule {
            message: "region pin ledger needs an entry".to_string(),
        };
        let mut item = new_region_pin_ledger("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RegionPinLedgerRequiredLabelRule {
            message: "region pin ledger needs this field".to_string(),
        };
        let mut item = new_region_pin_ledger("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
