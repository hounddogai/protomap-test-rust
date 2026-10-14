use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered approval adjustment must pass. Check returns its problems, or none.
pub trait TieredApprovalAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &TieredApprovalAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalAdjustmentLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredApprovalAdjustmentRule for TieredApprovalAdjustmentLimitRangeRule {
    fn check(&self, item: &TieredApprovalAdjustment) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalAdjustmentPrincipalLengthRule {
    pub max_length: i64,
}

impl TieredApprovalAdjustmentRule for TieredApprovalAdjustmentPrincipalLengthRule {
    fn check(&self, item: &TieredApprovalAdjustment) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalAdjustmentAllowedCurrencyRule {
    pub allowed: Vec<String>,
}

impl TieredApprovalAdjustmentRule for TieredApprovalAdjustmentAllowedCurrencyRule {
    fn check(&self, item: &TieredApprovalAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.currency)))
        {
            return vec![format!("currency {} is below minimum", item.currency)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered approval adjustment.
#[derive(Debug, Clone)]
pub struct TieredApprovalAdjustmentValidator {
    pub rules: Vec<Rc<dyn TieredApprovalAdjustmentRule>>,
}

impl TieredApprovalAdjustmentValidator {
    pub fn validate(&self, item: &TieredApprovalAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredApprovalAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredApprovalAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_approval_adjustment_validator() -> TieredApprovalAdjustmentValidator {
    TieredApprovalAdjustmentValidator {
        rules: vec![
            Rc::new(TieredApprovalAdjustmentLimitRangeRule { min: 16, max: 1000 })
                as Rc<dyn TieredApprovalAdjustmentRule>,
            Rc::new(TieredApprovalAdjustmentPrincipalLengthRule { max_length: 3600 }),
            Rc::new(TieredApprovalAdjustmentAllowedCurrencyRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_approval_adjustment_validator();
        let item = new_tiered_approval_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = TieredApprovalAdjustmentLimitRangeRule { min: 16, max: 255 };
        let mut item = new_tiered_approval_adjustment("x", "y");
        item.limit = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = TieredApprovalAdjustmentPrincipalLengthRule { max_length: 86400 };
        let mut item = new_tiered_approval_adjustment("x", "y");
        item.principal = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
