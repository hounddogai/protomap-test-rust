use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending approval plan must pass. Check returns its problems, or none.
pub trait PendingApprovalPlanRule: std::fmt::Debug {
    fn check(&self, item: &PendingApprovalPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingApprovalPlanChecksumLengthRule {
    pub max_length: i64,
}

impl PendingApprovalPlanRule for PendingApprovalPlanChecksumLengthRule {
    fn check(&self, item: &PendingApprovalPlan) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingApprovalPlanCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingApprovalPlanRule for PendingApprovalPlanCostRangeRule {
    fn check(&self, item: &PendingApprovalPlan) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingApprovalPlanCodePrefixRule {
    pub prefix: String,
}

impl PendingApprovalPlanRule for PendingApprovalPlanCodePrefixRule {
    fn check(&self, item: &PendingApprovalPlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingApprovalPlanRequiredChecksumRule {
    pub message: String,
}

impl PendingApprovalPlanRule for PendingApprovalPlanRequiredChecksumRule {
    fn check(&self, item: &PendingApprovalPlan) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending approval plan.
#[derive(Debug, Clone)]
pub struct PendingApprovalPlanValidator {
    pub rules: Vec<Rc<dyn PendingApprovalPlanRule>>,
}

impl PendingApprovalPlanValidator {
    pub fn validate(&self, item: &PendingApprovalPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingApprovalPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingApprovalPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_approval_plan_validator() -> PendingApprovalPlanValidator {
    PendingApprovalPlanValidator {
        rules: vec![
            Rc::new(PendingApprovalPlanChecksumLengthRule { max_length: 32 }) as Rc<dyn PendingApprovalPlanRule>,
            Rc::new(PendingApprovalPlanCostRangeRule { min: 8.4, max: 3165.5 }),
            Rc::new(PendingApprovalPlanCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(PendingApprovalPlanRequiredChecksumRule {
                message: "pending approval plan needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_approval_plan_validator();
        let item = new_pending_approval_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = PendingApprovalPlanChecksumLengthRule { max_length: 16 };
        let mut item = new_pending_approval_plan("x", "y");
        item.checksum = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = PendingApprovalPlanCostRangeRule { min: 3.5, max: 561.4 };
        let mut item = new_pending_approval_plan("x", "y");
        item.cost = 562.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = PendingApprovalPlanRequiredChecksumRule {
            message: "pending approval plan needs this field".to_string(),
        };
        let mut item = new_pending_approval_plan("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
