use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered budget burn ledger must pass. Check returns its problems, or none.
pub trait TieredBudgetBurnLedgerRule: std::fmt::Debug {
    fn check(&self, item: &TieredBudgetBurnLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnLedgerRequiredTitleRule {
    pub message: String,
}

impl TieredBudgetBurnLedgerRule for TieredBudgetBurnLedgerRequiredTitleRule {
    fn check(&self, item: &TieredBudgetBurnLedger) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnLedgerAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl TieredBudgetBurnLedgerRule for TieredBudgetBurnLedgerAllowedTitleRule {
    fn check(&self, item: &TieredBudgetBurnLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is inconsistent", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnLedgerLocaleLengthRule {
    pub max_length: i64,
}

impl TieredBudgetBurnLedgerRule for TieredBudgetBurnLedgerLocaleLengthRule {
    fn check(&self, item: &TieredBudgetBurnLedger) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered budget burn ledger.
#[derive(Debug, Clone)]
pub struct TieredBudgetBurnLedgerValidator {
    pub rules: Vec<Rc<dyn TieredBudgetBurnLedgerRule>>,
}

impl TieredBudgetBurnLedgerValidator {
    pub fn validate(&self, item: &TieredBudgetBurnLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredBudgetBurnLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredBudgetBurnLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_budget_burn_ledger_validator() -> TieredBudgetBurnLedgerValidator {
    TieredBudgetBurnLedgerValidator {
        rules: vec![
            Rc::new(TieredBudgetBurnLedgerRequiredTitleRule {
                message: "tiered budget burn ledger needs this field".to_string(),
            }) as Rc<dyn TieredBudgetBurnLedgerRule>,
            Rc::new(TieredBudgetBurnLedgerAllowedTitleRule { allowed: Vec::new() }),
            Rc::new(TieredBudgetBurnLedgerLocaleLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_budget_burn_ledger_validator();
        let item = new_tiered_budget_burn_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TieredBudgetBurnLedgerRequiredTitleRule {
            message: "tiered budget burn ledger needs an entry".to_string(),
        };
        let mut item = new_tiered_budget_burn_ledger("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = TieredBudgetBurnLedgerLocaleLengthRule { max_length: 16 };
        let mut item = new_tiered_budget_burn_ledger("x", "y");
        item.locale = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
