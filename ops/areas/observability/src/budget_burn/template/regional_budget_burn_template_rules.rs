use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional budget burn template must pass. Check returns its problems, or none.
pub trait RegionalBudgetBurnTemplateRule: std::fmt::Debug {
    fn check(&self, item: &RegionalBudgetBurnTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBudgetBurnTemplateLabelLengthRule {
    pub max_length: i64,
}

impl RegionalBudgetBurnTemplateRule for RegionalBudgetBurnTemplateLabelLengthRule {
    fn check(&self, item: &RegionalBudgetBurnTemplate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBudgetBurnTemplateCodePrefixRule {
    pub prefix: String,
}

impl RegionalBudgetBurnTemplateRule for RegionalBudgetBurnTemplateCodePrefixRule {
    fn check(&self, item: &RegionalBudgetBurnTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBudgetBurnTemplateThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalBudgetBurnTemplateRule for RegionalBudgetBurnTemplateThresholdRangeRule {
    fn check(&self, item: &RegionalBudgetBurnTemplate) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBudgetBurnTemplateRequiredLabelRule {
    pub message: String,
}

impl RegionalBudgetBurnTemplateRule for RegionalBudgetBurnTemplateRequiredLabelRule {
    fn check(&self, item: &RegionalBudgetBurnTemplate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBudgetBurnTemplateBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalBudgetBurnTemplateRule for RegionalBudgetBurnTemplateBatchSizeRangeRule {
    fn check(&self, item: &RegionalBudgetBurnTemplate) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional budget burn template.
#[derive(Debug, Clone)]
pub struct RegionalBudgetBurnTemplateValidator {
    pub rules: Vec<Rc<dyn RegionalBudgetBurnTemplateRule>>,
}

impl RegionalBudgetBurnTemplateValidator {
    pub fn validate(&self, item: &RegionalBudgetBurnTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalBudgetBurnTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalBudgetBurnTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_budget_burn_template_validator() -> RegionalBudgetBurnTemplateValidator {
    RegionalBudgetBurnTemplateValidator {
        rules: vec![
            Rc::new(RegionalBudgetBurnTemplateLabelLengthRule { max_length: 120 })
                as Rc<dyn RegionalBudgetBurnTemplateRule>,
            Rc::new(RegionalBudgetBurnTemplateCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(RegionalBudgetBurnTemplateThresholdRangeRule { min: 1.4, max: 4372.0 }),
            Rc::new(RegionalBudgetBurnTemplateRequiredLabelRule {
                message: "regional budget burn template needs this field".to_string(),
            }),
            Rc::new(RegionalBudgetBurnTemplateBatchSizeRangeRule { min: 32, max: 120 }),
        ],
    }
}
