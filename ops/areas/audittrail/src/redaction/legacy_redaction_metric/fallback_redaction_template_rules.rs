use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback redaction template must pass. Check returns its problems, or none.
pub trait FallbackRedactionTemplateRule: std::fmt::Debug {
    fn check(&self, item: &FallbackRedactionTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionTemplateApprovedGuardRule {
    pub limit: i64,
}

impl FallbackRedactionTemplateRule for FallbackRedactionTemplateApprovedGuardRule {
    fn check(&self, item: &FallbackRedactionTemplate) -> Vec<String> {
        if item.approved && (item.limit > self.limit) {
            return vec![format!("limit {} exceeds {} while approved", item.limit, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionTemplateAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl FallbackRedactionTemplateRule for FallbackRedactionTemplateAllowedCodeRule {
    fn check(&self, item: &FallbackRedactionTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionTemplateRequiredUnitRule {
    pub message: String,
}

impl FallbackRedactionTemplateRule for FallbackRedactionTemplateRequiredUnitRule {
    fn check(&self, item: &FallbackRedactionTemplate) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback redaction template.
#[derive(Debug, Clone)]
pub struct FallbackRedactionTemplateValidator {
    pub rules: Vec<Rc<dyn FallbackRedactionTemplateRule>>,
}

impl FallbackRedactionTemplateValidator {
    pub fn validate(&self, item: &FallbackRedactionTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackRedactionTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackRedactionTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_redaction_template_validator() -> FallbackRedactionTemplateValidator {
    FallbackRedactionTemplateValidator {
        rules: vec![
            Rc::new(FallbackRedactionTemplateApprovedGuardRule { limit: 32 }) as Rc<dyn FallbackRedactionTemplateRule>,
            Rc::new(FallbackRedactionTemplateAllowedCodeRule {
                allowed: vec!["strict".to_string(), "adaptive".to_string(), "fallback".to_string()],
            }),
            Rc::new(FallbackRedactionTemplateRequiredUnitRule {
                message: "fallback redaction template needs this field".to_string(),
            }),
        ],
    }
}
