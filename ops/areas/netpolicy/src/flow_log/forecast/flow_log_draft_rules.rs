use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a flow log draft must pass. Check returns its problems, or none.
pub trait FlowLogDraftRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogDraftCodePrefixRule {
    pub prefix: String,
}

impl FlowLogDraftRule for FlowLogDraftCodePrefixRule {
    fn check(&self, item: &FlowLogDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogDraftRequiredLabelRule {
    pub message: String,
}

impl FlowLogDraftRule for FlowLogDraftRequiredLabelRule {
    fn check(&self, item: &FlowLogDraft) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogDraftAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl FlowLogDraftRule for FlowLogDraftAllowedTenantIdRule {
    fn check(&self, item: &FlowLogDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is expired", item.tenant_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogDraftWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FlowLogDraftRule for FlowLogDraftWindowDaysRangeRule {
    fn check(&self, item: &FlowLogDraft) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a flow log draft.
#[derive(Debug, Clone)]
pub struct FlowLogDraftValidator {
    pub rules: Vec<Rc<dyn FlowLogDraftRule>>,
}

impl FlowLogDraftValidator {
    pub fn validate(&self, item: &FlowLogDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FlowLogDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_draft_validator() -> FlowLogDraftValidator {
    FlowLogDraftValidator {
        rules: vec![
            Rc::new(FlowLogDraftCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn FlowLogDraftRule>,
            Rc::new(FlowLogDraftRequiredLabelRule {
                message: "flow log draft needs an entry".to_string(),
            }),
            Rc::new(FlowLogDraftAllowedTenantIdRule { allowed: Vec::new() }),
            Rc::new(FlowLogDraftWindowDaysRangeRule { min: 80, max: 120 }),
        ],
    }
}
