use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a plan rollup must pass. Check returns its problems, or none.
pub trait PlanRollupRule: std::fmt::Debug {
    fn check(&self, item: &PlanRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanRollupAllowedCategoryRule {
    pub allowed: Vec<String>,
}

impl PlanRollupRule for PlanRollupAllowedCategoryRule {
    fn check(&self, item: &PlanRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.category)))
        {
            return vec![format!("category {} is unknown", item.category)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanRollupLocaleLengthRule {
    pub max_length: i64,
}

impl PlanRollupRule for PlanRollupLocaleLengthRule {
    fn check(&self, item: &PlanRollup) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanRollupRequiredCodeRule {
    pub message: String,
}

impl PlanRollupRule for PlanRollupRequiredCodeRule {
    fn check(&self, item: &PlanRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan rollup.
#[derive(Debug, Clone)]
pub struct PlanRollupValidator {
    pub rules: Vec<Rc<dyn PlanRollupRule>>,
}

impl PlanRollupValidator {
    pub fn validate(&self, item: &PlanRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PlanRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_rollup_validator() -> PlanRollupValidator {
    PlanRollupValidator {
        rules: vec![
            Rc::new(PlanRollupAllowedCategoryRule {
                allowed: vec!["seasonal".to_string()],
            }) as Rc<dyn PlanRollupRule>,
            Rc::new(PlanRollupLocaleLengthRule { max_length: 80 }),
            Rc::new(PlanRollupRequiredCodeRule {
                message: "plan rollup needs this field".to_string(),
            }),
        ],
    }
}
