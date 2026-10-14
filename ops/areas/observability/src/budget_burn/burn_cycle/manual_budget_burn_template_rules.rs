use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual budget burn template must pass. Check returns its problems, or none.
pub trait ManualBudgetBurnTemplateRule: std::fmt::Debug {
    fn check(&self, item: &ManualBudgetBurnTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnTemplatePinnedGuardRule {
    pub limit: i64,
}

impl ManualBudgetBurnTemplateRule for ManualBudgetBurnTemplatePinnedGuardRule {
    fn check(&self, item: &ManualBudgetBurnTemplate) -> Vec<String> {
        if item.pinned && (item.quantity > self.limit) {
            return vec![format!(
                "quantity {} exceeds {} while pinned",
                item.quantity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnTemplateQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualBudgetBurnTemplateRule for ManualBudgetBurnTemplateQuantityRangeRule {
    fn check(&self, item: &ManualBudgetBurnTemplate) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnTemplateAllowedClusterRule {
    pub allowed: Vec<String>,
}

impl ManualBudgetBurnTemplateRule for ManualBudgetBurnTemplateAllowedClusterRule {
    fn check(&self, item: &ManualBudgetBurnTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.cluster)))
        {
            return vec![format!("cluster {} is out of range", item.cluster)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual budget burn template.
#[derive(Debug, Clone)]
pub struct ManualBudgetBurnTemplateValidator {
    pub rules: Vec<Rc<dyn ManualBudgetBurnTemplateRule>>,
}

impl ManualBudgetBurnTemplateValidator {
    pub fn validate(&self, item: &ManualBudgetBurnTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualBudgetBurnTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualBudgetBurnTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_budget_burn_template_validator() -> ManualBudgetBurnTemplateValidator {
    ManualBudgetBurnTemplateValidator {
        rules: vec![
            Rc::new(ManualBudgetBurnTemplatePinnedGuardRule { limit: 80 }) as Rc<dyn ManualBudgetBurnTemplateRule>,
            Rc::new(ManualBudgetBurnTemplateQuantityRangeRule { min: 16, max: 86400 }),
            Rc::new(ManualBudgetBurnTemplateAllowedClusterRule {
                allowed: vec!["regional".to_string()],
            }),
        ],
    }
}
