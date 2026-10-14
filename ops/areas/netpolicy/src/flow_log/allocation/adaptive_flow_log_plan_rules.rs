use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive flow log plan must pass. Check returns its problems, or none.
pub trait AdaptiveFlowLogPlanRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveFlowLogPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFlowLogPlanTenantIdLengthRule {
    pub max_length: i64,
}

impl AdaptiveFlowLogPlanRule for AdaptiveFlowLogPlanTenantIdLengthRule {
    fn check(&self, item: &AdaptiveFlowLogPlan) -> Vec<String> {
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
pub struct AdaptiveFlowLogPlanQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveFlowLogPlanRule for AdaptiveFlowLogPlanQuotaUnitsRangeRule {
    fn check(&self, item: &AdaptiveFlowLogPlan) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFlowLogPlanCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveFlowLogPlanRule for AdaptiveFlowLogPlanCodeLengthRule {
    fn check(&self, item: &AdaptiveFlowLogPlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFlowLogPlanAllowedContactEmailRule {
    pub allowed: Vec<String>,
}

impl AdaptiveFlowLogPlanRule for AdaptiveFlowLogPlanAllowedContactEmailRule {
    fn check(&self, item: &AdaptiveFlowLogPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.contact_email)))
        {
            return vec![format!("contact email {} is not allowed", item.contact_email)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFlowLogPlanRequiredCodeRule {
    pub message: String,
}

impl AdaptiveFlowLogPlanRule for AdaptiveFlowLogPlanRequiredCodeRule {
    fn check(&self, item: &AdaptiveFlowLogPlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive flow log plan.
#[derive(Debug, Clone)]
pub struct AdaptiveFlowLogPlanValidator {
    pub rules: Vec<Rc<dyn AdaptiveFlowLogPlanRule>>,
}

impl AdaptiveFlowLogPlanValidator {
    pub fn validate(&self, item: &AdaptiveFlowLogPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveFlowLogPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveFlowLogPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_flow_log_plan_validator() -> AdaptiveFlowLogPlanValidator {
    AdaptiveFlowLogPlanValidator {
        rules: vec![
            Rc::new(AdaptiveFlowLogPlanTenantIdLengthRule { max_length: 16 }) as Rc<dyn AdaptiveFlowLogPlanRule>,
            Rc::new(AdaptiveFlowLogPlanQuotaUnitsRangeRule { min: 32, max: 120 }),
            Rc::new(AdaptiveFlowLogPlanCodeLengthRule { max_length: 86400 }),
            Rc::new(AdaptiveFlowLogPlanAllowedContactEmailRule {
                allowed: vec!["weighted".to_string(), "capped".to_string()],
            }),
            Rc::new(AdaptiveFlowLogPlanRequiredCodeRule {
                message: "adaptive flow log plan needs a value".to_string(),
            }),
        ],
    }
}
