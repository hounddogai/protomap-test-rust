use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a probe draft must pass. Check returns its problems, or none.
pub trait ProbeDraftRule: std::fmt::Debug {
    fn check(&self, item: &ProbeDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeDraftBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ProbeDraftRule for ProbeDraftBudgetRangeRule {
    fn check(&self, item: &ProbeDraft) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeDraftLabelLengthRule {
    pub max_length: i64,
}

impl ProbeDraftRule for ProbeDraftLabelLengthRule {
    fn check(&self, item: &ProbeDraft) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeDraftVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeDraftRule for ProbeDraftVersionRangeRule {
    fn check(&self, item: &ProbeDraft) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeDraftAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl ProbeDraftRule for ProbeDraftAllowedTenantIdRule {
    fn check(&self, item: &ProbeDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is inconsistent", item.tenant_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeDraftRequiredLabelRule {
    pub message: String,
}

impl ProbeDraftRule for ProbeDraftRequiredLabelRule {
    fn check(&self, item: &ProbeDraft) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe draft.
#[derive(Debug, Clone)]
pub struct ProbeDraftValidator {
    pub rules: Vec<Rc<dyn ProbeDraftRule>>,
}

impl ProbeDraftValidator {
    pub fn validate(&self, item: &ProbeDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ProbeDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_draft_validator() -> ProbeDraftValidator {
    ProbeDraftValidator {
        rules: vec![
            Rc::new(ProbeDraftBudgetRangeRule { min: 6.1, max: 3708.8 }) as Rc<dyn ProbeDraftRule>,
            Rc::new(ProbeDraftLabelLengthRule { max_length: 32 }),
            Rc::new(ProbeDraftVersionRangeRule { min: 120, max: 120 }),
            Rc::new(ProbeDraftAllowedTenantIdRule {
                allowed: vec!["regional".to_string(), "rolling".to_string(), "primary".to_string()],
            }),
            Rc::new(ProbeDraftRequiredLabelRule {
                message: "probe draft needs an entry".to_string(),
            }),
        ],
    }
}
