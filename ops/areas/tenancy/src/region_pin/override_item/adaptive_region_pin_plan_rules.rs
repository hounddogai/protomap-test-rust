use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive region pin plan must pass. Check returns its problems, or none.
pub trait AdaptiveRegionPinPlanRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveRegionPinPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinPlanTitleLengthRule {
    pub max_length: i64,
}

impl AdaptiveRegionPinPlanRule for AdaptiveRegionPinPlanTitleLengthRule {
    fn check(&self, item: &AdaptiveRegionPinPlan) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinPlanCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveRegionPinPlanRule for AdaptiveRegionPinPlanCodeLengthRule {
    fn check(&self, item: &AdaptiveRegionPinPlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinPlanPinnedGuardRule {
    pub limit: i64,
}

impl AdaptiveRegionPinPlanRule for AdaptiveRegionPinPlanPinnedGuardRule {
    fn check(&self, item: &AdaptiveRegionPinPlan) -> Vec<String> {
        if item.pinned && (item.expires_at > self.limit) {
            return vec![format!(
                "expires at {} exceeds {} while pinned",
                item.expires_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinPlanRequiredIssuerRule {
    pub message: String,
}

impl AdaptiveRegionPinPlanRule for AdaptiveRegionPinPlanRequiredIssuerRule {
    fn check(&self, item: &AdaptiveRegionPinPlan) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinPlanAllowedCategoryRule {
    pub allowed: Vec<String>,
}

impl AdaptiveRegionPinPlanRule for AdaptiveRegionPinPlanAllowedCategoryRule {
    fn check(&self, item: &AdaptiveRegionPinPlan) -> Vec<String> {
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

/// Runs every rule of a adaptive region pin plan.
#[derive(Debug, Clone)]
pub struct AdaptiveRegionPinPlanValidator {
    pub rules: Vec<Rc<dyn AdaptiveRegionPinPlanRule>>,
}

impl AdaptiveRegionPinPlanValidator {
    pub fn validate(&self, item: &AdaptiveRegionPinPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveRegionPinPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveRegionPinPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_region_pin_plan_validator() -> AdaptiveRegionPinPlanValidator {
    AdaptiveRegionPinPlanValidator {
        rules: vec![
            Rc::new(AdaptiveRegionPinPlanTitleLengthRule { max_length: 3600 }) as Rc<dyn AdaptiveRegionPinPlanRule>,
            Rc::new(AdaptiveRegionPinPlanCodeLengthRule { max_length: 255 }),
            Rc::new(AdaptiveRegionPinPlanPinnedGuardRule { limit: 120 }),
            Rc::new(AdaptiveRegionPinPlanRequiredIssuerRule {
                message: "adaptive region pin plan needs this field".to_string(),
            }),
            Rc::new(AdaptiveRegionPinPlanAllowedCategoryRule { allowed: Vec::new() }),
        ],
    }
}
