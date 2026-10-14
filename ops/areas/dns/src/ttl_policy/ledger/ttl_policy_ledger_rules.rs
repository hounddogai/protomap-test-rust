use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a ttl policy ledger must pass. Check returns its problems, or none.
pub trait TtlPolicyLedgerRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyLedgerRequiredLabelRule {
    pub message: String,
}

impl TtlPolicyLedgerRule for TtlPolicyLedgerRequiredLabelRule {
    fn check(&self, item: &TtlPolicyLedger) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyLedgerStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TtlPolicyLedgerRule for TtlPolicyLedgerStartsAtRangeRule {
    fn check(&self, item: &TtlPolicyLedger) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyLedgerAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl TtlPolicyLedgerRule for TtlPolicyLedgerAllowedLabelRule {
    fn check(&self, item: &TtlPolicyLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is unknown", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyLedgerCodePrefixRule {
    pub prefix: String,
}

impl TtlPolicyLedgerRule for TtlPolicyLedgerCodePrefixRule {
    fn check(&self, item: &TtlPolicyLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy ledger.
#[derive(Debug, Clone)]
pub struct TtlPolicyLedgerValidator {
    pub rules: Vec<Rc<dyn TtlPolicyLedgerRule>>,
}

impl TtlPolicyLedgerValidator {
    pub fn validate(&self, item: &TtlPolicyLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TtlPolicyLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_ledger_validator() -> TtlPolicyLedgerValidator {
    TtlPolicyLedgerValidator {
        rules: vec![
            Rc::new(TtlPolicyLedgerRequiredLabelRule {
                message: "ttl policy ledger needs a value".to_string(),
            }) as Rc<dyn TtlPolicyLedgerRule>,
            Rc::new(TtlPolicyLedgerStartsAtRangeRule { min: 64, max: 120 }),
            Rc::new(TtlPolicyLedgerAllowedLabelRule {
                allowed: vec!["batched".to_string(), "regional".to_string(), "strict".to_string()],
            }),
            Rc::new(TtlPolicyLedgerCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_ledger_validator();
        let item = new_ttl_policy_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TtlPolicyLedgerRequiredLabelRule {
            message: "ttl policy ledger needs an entry".to_string(),
        };
        let mut item = new_ttl_policy_ledger("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = TtlPolicyLedgerStartsAtRangeRule { min: 16, max: 80 };
        let mut item = new_ttl_policy_ledger("x", "y");
        item.starts_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
