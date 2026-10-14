use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary dashboard template must pass. Check returns its problems, or none.
pub trait PrimaryDashboardTemplateRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryDashboardTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDashboardTemplateRequiredKeyIdRule {
    pub message: String,
}

impl PrimaryDashboardTemplateRule for PrimaryDashboardTemplateRequiredKeyIdRule {
    fn check(&self, item: &PrimaryDashboardTemplate) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDashboardTemplateKeyIdLengthRule {
    pub max_length: i64,
}

impl PrimaryDashboardTemplateRule for PrimaryDashboardTemplateKeyIdLengthRule {
    fn check(&self, item: &PrimaryDashboardTemplate) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDashboardTemplateAllowedIssuerRule {
    pub allowed: Vec<String>,
}

impl PrimaryDashboardTemplateRule for PrimaryDashboardTemplateAllowedIssuerRule {
    fn check(&self, item: &PrimaryDashboardTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.issuer))) {
            return vec![format!("issuer {} is inconsistent", item.issuer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDashboardTemplateCodePrefixRule {
    pub prefix: String,
}

impl PrimaryDashboardTemplateRule for PrimaryDashboardTemplateCodePrefixRule {
    fn check(&self, item: &PrimaryDashboardTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDashboardTemplateTitleLengthRule {
    pub max_length: i64,
}

impl PrimaryDashboardTemplateRule for PrimaryDashboardTemplateTitleLengthRule {
    fn check(&self, item: &PrimaryDashboardTemplate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDashboardTemplateQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryDashboardTemplateRule for PrimaryDashboardTemplateQuantityRangeRule {
    fn check(&self, item: &PrimaryDashboardTemplate) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary dashboard template.
#[derive(Debug, Clone)]
pub struct PrimaryDashboardTemplateValidator {
    pub rules: Vec<Rc<dyn PrimaryDashboardTemplateRule>>,
}

impl PrimaryDashboardTemplateValidator {
    pub fn validate(&self, item: &PrimaryDashboardTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryDashboardTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryDashboardTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_dashboard_template_validator() -> PrimaryDashboardTemplateValidator {
    PrimaryDashboardTemplateValidator {
        rules: vec![
            Rc::new(PrimaryDashboardTemplateRequiredKeyIdRule {
                message: "primary dashboard template needs an entry".to_string(),
            }) as Rc<dyn PrimaryDashboardTemplateRule>,
            Rc::new(PrimaryDashboardTemplateKeyIdLengthRule { max_length: 120 }),
            Rc::new(PrimaryDashboardTemplateAllowedIssuerRule {
                allowed: vec!["tiered".to_string(), "pending".to_string(), "primary".to_string()],
            }),
            Rc::new(PrimaryDashboardTemplateCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(PrimaryDashboardTemplateTitleLengthRule { max_length: 120 }),
            Rc::new(PrimaryDashboardTemplateQuantityRangeRule { min: 120, max: 255 }),
        ],
    }
}
