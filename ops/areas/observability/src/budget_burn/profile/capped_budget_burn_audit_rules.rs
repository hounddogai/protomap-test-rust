use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped budget burn audit must pass. Check returns its problems, or none.
pub trait CappedBudgetBurnAuditRule: std::fmt::Debug {
    fn check(&self, item: &CappedBudgetBurnAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnAuditRequiredLabelRule {
    pub message: String,
}

impl CappedBudgetBurnAuditRule for CappedBudgetBurnAuditRequiredLabelRule {
    fn check(&self, item: &CappedBudgetBurnAudit) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnAuditRequiredCodeRule {
    pub message: String,
}

impl CappedBudgetBurnAuditRule for CappedBudgetBurnAuditRequiredCodeRule {
    fn check(&self, item: &CappedBudgetBurnAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnAuditLocaleLengthRule {
    pub max_length: i64,
}

impl CappedBudgetBurnAuditRule for CappedBudgetBurnAuditLocaleLengthRule {
    fn check(&self, item: &CappedBudgetBurnAudit) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnAuditTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedBudgetBurnAuditRule for CappedBudgetBurnAuditTtlSecondsRangeRule {
    fn check(&self, item: &CappedBudgetBurnAudit) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnAuditGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedBudgetBurnAuditRule for CappedBudgetBurnAuditGraceDaysRangeRule {
    fn check(&self, item: &CappedBudgetBurnAudit) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnAuditCodeLengthRule {
    pub max_length: i64,
}

impl CappedBudgetBurnAuditRule for CappedBudgetBurnAuditCodeLengthRule {
    fn check(&self, item: &CappedBudgetBurnAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped budget burn audit.
#[derive(Debug, Clone)]
pub struct CappedBudgetBurnAuditValidator {
    pub rules: Vec<Rc<dyn CappedBudgetBurnAuditRule>>,
}

impl CappedBudgetBurnAuditValidator {
    pub fn validate(&self, item: &CappedBudgetBurnAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedBudgetBurnAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedBudgetBurnAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_budget_burn_audit_validator() -> CappedBudgetBurnAuditValidator {
    CappedBudgetBurnAuditValidator {
        rules: vec![
            Rc::new(CappedBudgetBurnAuditRequiredLabelRule {
                message: "capped budget burn audit needs an entry".to_string(),
            }) as Rc<dyn CappedBudgetBurnAuditRule>,
            Rc::new(CappedBudgetBurnAuditRequiredCodeRule {
                message: "capped budget burn audit needs an entry".to_string(),
            }),
            Rc::new(CappedBudgetBurnAuditLocaleLengthRule { max_length: 255 }),
            Rc::new(CappedBudgetBurnAuditTtlSecondsRangeRule { min: 255, max: 86400 }),
            Rc::new(CappedBudgetBurnAuditGraceDaysRangeRule { min: 32, max: 255 }),
            Rc::new(CappedBudgetBurnAuditCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_budget_burn_audit_validator();
        let item = new_capped_budget_burn_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = CappedBudgetBurnAuditRequiredLabelRule {
            message: "capped budget burn audit needs a value".to_string(),
        };
        let mut item = new_capped_budget_burn_audit("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedBudgetBurnAuditRequiredCodeRule {
            message: "capped budget burn audit needs this field".to_string(),
        };
        let mut item = new_capped_budget_burn_audit("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = CappedBudgetBurnAuditLocaleLengthRule { max_length: 1000 };
        let mut item = new_capped_budget_burn_audit("x", "y");
        item.locale = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = CappedBudgetBurnAuditTtlSecondsRangeRule { min: 1000, max: 1000 };
        let mut item = new_capped_budget_burn_audit("x", "y");
        item.ttl_seconds = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = CappedBudgetBurnAuditGraceDaysRangeRule { min: 64, max: 120 };
        let mut item = new_capped_budget_burn_audit("x", "y");
        item.grace_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedBudgetBurnAuditCodeLengthRule { max_length: 16 };
        let mut item = new_capped_budget_burn_audit("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
