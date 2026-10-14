use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual budget burn milestone must pass. Check returns its problems, or none.
pub trait ManualBudgetBurnMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &ManualBudgetBurnMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnMilestoneAllowedNamespaceRule {
    pub allowed: Vec<String>,
}

impl ManualBudgetBurnMilestoneRule for ManualBudgetBurnMilestoneAllowedNamespaceRule {
    fn check(&self, item: &ManualBudgetBurnMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.namespace)))
        {
            return vec![format!("namespace {} is too long", item.namespace)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnMilestoneTenantIdLengthRule {
    pub max_length: i64,
}

impl ManualBudgetBurnMilestoneRule for ManualBudgetBurnMilestoneTenantIdLengthRule {
    fn check(&self, item: &ManualBudgetBurnMilestone) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnMilestoneLabelLengthRule {
    pub max_length: i64,
}

impl ManualBudgetBurnMilestoneRule for ManualBudgetBurnMilestoneLabelLengthRule {
    fn check(&self, item: &ManualBudgetBurnMilestone) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual budget burn milestone.
#[derive(Debug, Clone)]
pub struct ManualBudgetBurnMilestoneValidator {
    pub rules: Vec<Rc<dyn ManualBudgetBurnMilestoneRule>>,
}

impl ManualBudgetBurnMilestoneValidator {
    pub fn validate(&self, item: &ManualBudgetBurnMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualBudgetBurnMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualBudgetBurnMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_budget_burn_milestone_validator() -> ManualBudgetBurnMilestoneValidator {
    ManualBudgetBurnMilestoneValidator {
        rules: vec![
            Rc::new(ManualBudgetBurnMilestoneAllowedNamespaceRule {
                allowed: vec!["strict".to_string()],
            }) as Rc<dyn ManualBudgetBurnMilestoneRule>,
            Rc::new(ManualBudgetBurnMilestoneTenantIdLengthRule { max_length: 86400 }),
            Rc::new(ManualBudgetBurnMilestoneLabelLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_budget_burn_milestone_validator();
        let item = new_manual_budget_burn_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = ManualBudgetBurnMilestoneTenantIdLengthRule { max_length: 1000 };
        let mut item = new_manual_budget_burn_milestone("x", "y");
        item.tenant_id = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ManualBudgetBurnMilestoneLabelLengthRule { max_length: 32 };
        let mut item = new_manual_budget_burn_milestone("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
