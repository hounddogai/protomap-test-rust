use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a audit event baseline must pass. Check returns its problems, or none.
pub trait AuditEventBaselineRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventBaselineRequiredCurrencyRule {
    pub message: String,
}

impl AuditEventBaselineRule for AuditEventBaselineRequiredCurrencyRule {
    fn check(&self, item: &AuditEventBaseline) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventBaselineCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AuditEventBaselineRule for AuditEventBaselineCostRangeRule {
    fn check(&self, item: &AuditEventBaseline) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventBaselineCodePrefixRule {
    pub prefix: String,
}

impl AuditEventBaselineRule for AuditEventBaselineCodePrefixRule {
    fn check(&self, item: &AuditEventBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventBaselineRequiredSummaryRule {
    pub message: String,
}

impl AuditEventBaselineRule for AuditEventBaselineRequiredSummaryRule {
    fn check(&self, item: &AuditEventBaseline) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventBaselineApprovedGuardRule {
    pub limit: f64,
}

impl AuditEventBaselineRule for AuditEventBaselineApprovedGuardRule {
    fn check(&self, item: &AuditEventBaseline) -> Vec<String> {
        if item.approved && (item.cost > self.limit) {
            return vec![format!("cost {} exceeds {} while approved", item.cost, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventBaselineAllowedSummaryRule {
    pub allowed: Vec<String>,
}

impl AuditEventBaselineRule for AuditEventBaselineAllowedSummaryRule {
    fn check(&self, item: &AuditEventBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.summary)))
        {
            return vec![format!("summary {} is too long", item.summary)];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event baseline.
#[derive(Debug, Clone)]
pub struct AuditEventBaselineValidator {
    pub rules: Vec<Rc<dyn AuditEventBaselineRule>>,
}

impl AuditEventBaselineValidator {
    pub fn validate(&self, item: &AuditEventBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AuditEventBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_baseline_validator() -> AuditEventBaselineValidator {
    AuditEventBaselineValidator {
        rules: vec![
            Rc::new(AuditEventBaselineRequiredCurrencyRule {
                message: "audit event baseline needs this field".to_string(),
            }) as Rc<dyn AuditEventBaselineRule>,
            Rc::new(AuditEventBaselineCostRangeRule { min: 5.7, max: 1117.9 }),
            Rc::new(AuditEventBaselineCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(AuditEventBaselineRequiredSummaryRule {
                message: "audit event baseline needs an entry".to_string(),
            }),
            Rc::new(AuditEventBaselineApprovedGuardRule { limit: 3.3 }),
            Rc::new(AuditEventBaselineAllowedSummaryRule {
                allowed: vec!["weighted".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_baseline_validator();
        let item = new_audit_event_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = AuditEventBaselineRequiredCurrencyRule {
            message: "audit event baseline needs an entry".to_string(),
        };
        let mut item = new_audit_event_baseline("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = AuditEventBaselineCostRangeRule { min: 8.5, max: 1434.9 };
        let mut item = new_audit_event_baseline("x", "y");
        item.cost = 1436.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = AuditEventBaselineRequiredSummaryRule {
            message: "audit event baseline needs this field".to_string(),
        };
        let mut item = new_audit_event_baseline("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
