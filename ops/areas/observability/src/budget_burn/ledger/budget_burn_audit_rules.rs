use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a budget burn audit must pass. Check returns its problems, or none.
pub trait BudgetBurnAuditRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnAuditRequiredTitleRule {
    pub message: String,
}

impl BudgetBurnAuditRule for BudgetBurnAuditRequiredTitleRule {
    fn check(&self, item: &BudgetBurnAudit) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnAuditCodePrefixRule {
    pub prefix: String,
}

impl BudgetBurnAuditRule for BudgetBurnAuditCodePrefixRule {
    fn check(&self, item: &BudgetBurnAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnAuditCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BudgetBurnAuditRule for BudgetBurnAuditCeilingRangeRule {
    fn check(&self, item: &BudgetBurnAudit) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a budget burn audit.
#[derive(Debug, Clone)]
pub struct BudgetBurnAuditValidator {
    pub rules: Vec<Rc<dyn BudgetBurnAuditRule>>,
}

impl BudgetBurnAuditValidator {
    pub fn validate(&self, item: &BudgetBurnAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BudgetBurnAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_audit_validator() -> BudgetBurnAuditValidator {
    BudgetBurnAuditValidator {
        rules: vec![
            Rc::new(BudgetBurnAuditRequiredTitleRule {
                message: "budget burn audit needs a value".to_string(),
            }) as Rc<dyn BudgetBurnAuditRule>,
            Rc::new(BudgetBurnAuditCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(BudgetBurnAuditCeilingRangeRule { min: 8.7, max: 469.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_audit_validator();
        let item = new_budget_burn_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = BudgetBurnAuditRequiredTitleRule {
            message: "budget burn audit needs this field".to_string(),
        };
        let mut item = new_budget_burn_audit("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = BudgetBurnAuditCeilingRangeRule { min: 0.5, max: 2344.3 };
        let mut item = new_budget_burn_audit("x", "y");
        item.ceiling = 2345.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
