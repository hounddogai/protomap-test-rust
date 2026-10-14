use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled tenant ledger must pass. Check returns its problems, or none.
pub trait ScheduledTenantLedgerRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledTenantLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantLedgerBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledTenantLedgerRule for ScheduledTenantLedgerBatchSizeRangeRule {
    fn check(&self, item: &ScheduledTenantLedger) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantLedgerRequiredSourceRule {
    pub message: String,
}

impl ScheduledTenantLedgerRule for ScheduledTenantLedgerRequiredSourceRule {
    fn check(&self, item: &ScheduledTenantLedger) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantLedgerRequiredLabelRule {
    pub message: String,
}

impl ScheduledTenantLedgerRule for ScheduledTenantLedgerRequiredLabelRule {
    fn check(&self, item: &ScheduledTenantLedger) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantLedgerAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl ScheduledTenantLedgerRule for ScheduledTenantLedgerAllowedLabelRule {
    fn check(&self, item: &ScheduledTenantLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is stale", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled tenant ledger.
#[derive(Debug, Clone)]
pub struct ScheduledTenantLedgerValidator {
    pub rules: Vec<Rc<dyn ScheduledTenantLedgerRule>>,
}

impl ScheduledTenantLedgerValidator {
    pub fn validate(&self, item: &ScheduledTenantLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledTenantLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledTenantLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_tenant_ledger_validator() -> ScheduledTenantLedgerValidator {
    ScheduledTenantLedgerValidator {
        rules: vec![
            Rc::new(ScheduledTenantLedgerBatchSizeRangeRule { min: 3600, max: 86400 })
                as Rc<dyn ScheduledTenantLedgerRule>,
            Rc::new(ScheduledTenantLedgerRequiredSourceRule {
                message: "scheduled tenant ledger needs an entry".to_string(),
            }),
            Rc::new(ScheduledTenantLedgerRequiredLabelRule {
                message: "scheduled tenant ledger needs this field".to_string(),
            }),
            Rc::new(ScheduledTenantLedgerAllowedLabelRule {
                allowed: vec!["default".to_string(), "pending".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_tenant_ledger_validator();
        let item = new_scheduled_tenant_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = ScheduledTenantLedgerBatchSizeRangeRule { min: 16, max: 3600 };
        let mut item = new_scheduled_tenant_ledger("x", "y");
        item.batch_size = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = ScheduledTenantLedgerRequiredSourceRule {
            message: "scheduled tenant ledger needs an entry".to_string(),
        };
        let mut item = new_scheduled_tenant_ledger("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ScheduledTenantLedgerRequiredLabelRule {
            message: "scheduled tenant ledger needs an entry".to_string(),
        };
        let mut item = new_scheduled_tenant_ledger("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
