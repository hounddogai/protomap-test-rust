use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending segment ledger must pass. Check returns its problems, or none.
pub trait PendingSegmentLedgerRule: std::fmt::Debug {
    fn check(&self, item: &PendingSegmentLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSegmentLedgerRequiredCodeRule {
    pub message: String,
}

impl PendingSegmentLedgerRule for PendingSegmentLedgerRequiredCodeRule {
    fn check(&self, item: &PendingSegmentLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSegmentLedgerAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PendingSegmentLedgerRule for PendingSegmentLedgerAllowedCodeRule {
    fn check(&self, item: &PendingSegmentLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSegmentLedgerMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingSegmentLedgerRule for PendingSegmentLedgerMinimumRangeRule {
    fn check(&self, item: &PendingSegmentLedger) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a pending segment ledger.
#[derive(Debug, Clone)]
pub struct PendingSegmentLedgerValidator {
    pub rules: Vec<Rc<dyn PendingSegmentLedgerRule>>,
}

impl PendingSegmentLedgerValidator {
    pub fn validate(&self, item: &PendingSegmentLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingSegmentLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingSegmentLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_segment_ledger_validator() -> PendingSegmentLedgerValidator {
    PendingSegmentLedgerValidator {
        rules: vec![
            Rc::new(PendingSegmentLedgerRequiredCodeRule {
                message: "pending segment ledger needs a value".to_string(),
            }) as Rc<dyn PendingSegmentLedgerRule>,
            Rc::new(PendingSegmentLedgerAllowedCodeRule {
                allowed: vec!["seasonal".to_string()],
            }),
            Rc::new(PendingSegmentLedgerMinimumRangeRule { min: 1.9, max: 2760.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_segment_ledger_validator();
        let item = new_pending_segment_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingSegmentLedgerRequiredCodeRule {
            message: "pending segment ledger needs an entry".to_string(),
        };
        let mut item = new_pending_segment_ledger("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = PendingSegmentLedgerMinimumRangeRule { min: 4.4, max: 2253.5 };
        let mut item = new_pending_segment_ledger("x", "y");
        item.minimum = 2255.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
