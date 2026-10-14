use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback plan template must pass. Check returns its problems, or none.
pub trait FallbackPlanTemplateRule: std::fmt::Debug {
    fn check(&self, item: &FallbackPlanTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanTemplateLabelLengthRule {
    pub max_length: i64,
}

impl FallbackPlanTemplateRule for FallbackPlanTemplateLabelLengthRule {
    fn check(&self, item: &FallbackPlanTemplate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanTemplateRequiredLabelRule {
    pub message: String,
}

impl FallbackPlanTemplateRule for FallbackPlanTemplateRequiredLabelRule {
    fn check(&self, item: &FallbackPlanTemplate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanTemplateCodeLengthRule {
    pub max_length: i64,
}

impl FallbackPlanTemplateRule for FallbackPlanTemplateCodeLengthRule {
    fn check(&self, item: &FallbackPlanTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanTemplateTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackPlanTemplateRule for FallbackPlanTemplateTtlSecondsRangeRule {
    fn check(&self, item: &FallbackPlanTemplate) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback plan template.
#[derive(Debug, Clone)]
pub struct FallbackPlanTemplateValidator {
    pub rules: Vec<Rc<dyn FallbackPlanTemplateRule>>,
}

impl FallbackPlanTemplateValidator {
    pub fn validate(&self, item: &FallbackPlanTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackPlanTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackPlanTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_plan_template_validator() -> FallbackPlanTemplateValidator {
    FallbackPlanTemplateValidator {
        rules: vec![
            Rc::new(FallbackPlanTemplateLabelLengthRule { max_length: 16 }) as Rc<dyn FallbackPlanTemplateRule>,
            Rc::new(FallbackPlanTemplateRequiredLabelRule {
                message: "fallback plan template needs this field".to_string(),
            }),
            Rc::new(FallbackPlanTemplateCodeLengthRule { max_length: 32 }),
            Rc::new(FallbackPlanTemplateTtlSecondsRangeRule { min: 64, max: 64 }),
        ],
    }
}
