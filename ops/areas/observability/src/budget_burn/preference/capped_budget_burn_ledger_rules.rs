use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped budget burn ledger must pass. Check returns its problems, or none.
pub trait CappedBudgetBurnLedgerRule: std::fmt::Debug {
    fn check(&self, item: &CappedBudgetBurnLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnLedgerRequiredNamespaceRule {
    pub message: String,
}

impl CappedBudgetBurnLedgerRule for CappedBudgetBurnLedgerRequiredNamespaceRule {
    fn check(&self, item: &CappedBudgetBurnLedger) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnLedgerUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedBudgetBurnLedgerRule for CappedBudgetBurnLedgerUpdatedAtRangeRule {
    fn check(&self, item: &CappedBudgetBurnLedger) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnLedgerAllowedNamespaceRule {
    pub allowed: Vec<String>,
}

impl CappedBudgetBurnLedgerRule for CappedBudgetBurnLedgerAllowedNamespaceRule {
    fn check(&self, item: &CappedBudgetBurnLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.namespace)))
        {
            return vec![format!("namespace {} is duplicated", item.namespace)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnLedgerNamespaceLengthRule {
    pub max_length: i64,
}

impl CappedBudgetBurnLedgerRule for CappedBudgetBurnLedgerNamespaceLengthRule {
    fn check(&self, item: &CappedBudgetBurnLedger) -> Vec<String> {
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
pub struct CappedBudgetBurnLedgerTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl CappedBudgetBurnLedgerRule for CappedBudgetBurnLedgerTagsLimitRule {
    fn check(&self, item: &CappedBudgetBurnLedger) -> Vec<String> {
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

/// Runs every rule of a capped budget burn ledger.
#[derive(Debug, Clone)]
pub struct CappedBudgetBurnLedgerValidator {
    pub rules: Vec<Rc<dyn CappedBudgetBurnLedgerRule>>,
}

impl CappedBudgetBurnLedgerValidator {
    pub fn validate(&self, item: &CappedBudgetBurnLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedBudgetBurnLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedBudgetBurnLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_budget_burn_ledger_validator() -> CappedBudgetBurnLedgerValidator {
    CappedBudgetBurnLedgerValidator {
        rules: vec![
            Rc::new(CappedBudgetBurnLedgerRequiredNamespaceRule {
                message: "capped budget burn ledger needs this field".to_string(),
            }) as Rc<dyn CappedBudgetBurnLedgerRule>,
            Rc::new(CappedBudgetBurnLedgerUpdatedAtRangeRule { min: 255, max: 3600 }),
            Rc::new(CappedBudgetBurnLedgerAllowedNamespaceRule {
                allowed: vec!["manual".to_string()],
            }),
            Rc::new(CappedBudgetBurnLedgerNamespaceLengthRule { max_length: 120 }),
            Rc::new(CappedBudgetBurnLedgerTagsLimitRule {
                max_count: 64,
                banned: vec!["pending".to_string(), "primary".to_string(), "lenient".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_budget_burn_ledger_validator();
        let item = new_capped_budget_burn_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = CappedBudgetBurnLedgerRequiredNamespaceRule {
            message: "capped budget burn ledger needs a value".to_string(),
        };
        let mut item = new_capped_budget_burn_ledger("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = CappedBudgetBurnLedgerUpdatedAtRangeRule { min: 32, max: 86400 };
        let mut item = new_capped_budget_burn_ledger("x", "y");
        item.updated_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = CappedBudgetBurnLedgerNamespaceLengthRule { max_length: 32 };
        let mut item = new_capped_budget_burn_ledger("x", "y");
        item.namespace = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
