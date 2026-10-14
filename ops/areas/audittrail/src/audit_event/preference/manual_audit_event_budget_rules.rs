use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual audit event budget must pass. Check returns its problems, or none.
pub trait ManualAuditEventBudgetRule: std::fmt::Debug {
    fn check(&self, item: &ManualAuditEventBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAuditEventBudgetRequiredLabelRule {
    pub message: String,
}

impl ManualAuditEventBudgetRule for ManualAuditEventBudgetRequiredLabelRule {
    fn check(&self, item: &ManualAuditEventBudget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAuditEventBudgetRequiredRegionRule {
    pub message: String,
}

impl ManualAuditEventBudgetRule for ManualAuditEventBudgetRequiredRegionRule {
    fn check(&self, item: &ManualAuditEventBudget) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAuditEventBudgetRequiredClusterRule {
    pub message: String,
}

impl ManualAuditEventBudgetRule for ManualAuditEventBudgetRequiredClusterRule {
    fn check(&self, item: &ManualAuditEventBudget) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAuditEventBudgetClusterLengthRule {
    pub max_length: i64,
}

impl ManualAuditEventBudgetRule for ManualAuditEventBudgetClusterLengthRule {
    fn check(&self, item: &ManualAuditEventBudget) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAuditEventBudgetDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualAuditEventBudgetRule for ManualAuditEventBudgetDiscountRangeRule {
    fn check(&self, item: &ManualAuditEventBudget) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a manual audit event budget.
#[derive(Debug, Clone)]
pub struct ManualAuditEventBudgetValidator {
    pub rules: Vec<Rc<dyn ManualAuditEventBudgetRule>>,
}

impl ManualAuditEventBudgetValidator {
    pub fn validate(&self, item: &ManualAuditEventBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualAuditEventBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualAuditEventBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_audit_event_budget_validator() -> ManualAuditEventBudgetValidator {
    ManualAuditEventBudgetValidator {
        rules: vec![
            Rc::new(ManualAuditEventBudgetRequiredLabelRule {
                message: "manual audit event budget needs a value".to_string(),
            }) as Rc<dyn ManualAuditEventBudgetRule>,
            Rc::new(ManualAuditEventBudgetRequiredRegionRule {
                message: "manual audit event budget needs an entry".to_string(),
            }),
            Rc::new(ManualAuditEventBudgetRequiredClusterRule {
                message: "manual audit event budget needs this field".to_string(),
            }),
            Rc::new(ManualAuditEventBudgetClusterLengthRule { max_length: 80 }),
            Rc::new(ManualAuditEventBudgetDiscountRangeRule { min: 0.4, max: 4764.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_audit_event_budget_validator();
        let item = new_manual_audit_event_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ManualAuditEventBudgetRequiredLabelRule {
            message: "manual audit event budget needs this field".to_string(),
        };
        let mut item = new_manual_audit_event_budget("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = ManualAuditEventBudgetRequiredRegionRule {
            message: "manual audit event budget needs this field".to_string(),
        };
        let mut item = new_manual_audit_event_budget("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = ManualAuditEventBudgetRequiredClusterRule {
            message: "manual audit event budget needs this field".to_string(),
        };
        let mut item = new_manual_audit_event_budget("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = ManualAuditEventBudgetClusterLengthRule { max_length: 16 };
        let mut item = new_manual_audit_event_budget("x", "y");
        item.cluster = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = ManualAuditEventBudgetDiscountRangeRule { min: 7.0, max: 3071.7 };
        let mut item = new_manual_audit_event_budget("x", "y");
        item.discount = 3073.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
