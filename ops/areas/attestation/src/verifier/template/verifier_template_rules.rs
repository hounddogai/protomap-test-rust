use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier template must pass. Check returns its problems, or none.
pub trait VerifierTemplateRule: std::fmt::Debug {
    fn check(&self, item: &VerifierTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierTemplateRequiredCodeRule {
    pub message: String,
}

impl VerifierTemplateRule for VerifierTemplateRequiredCodeRule {
    fn check(&self, item: &VerifierTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierTemplatePinnedGuardRule {
    pub limit: i64,
}

impl VerifierTemplateRule for VerifierTemplatePinnedGuardRule {
    fn check(&self, item: &VerifierTemplate) -> Vec<String> {
        if item.pinned && (item.priority > self.limit) {
            return vec![format!(
                "priority {} exceeds {} while pinned",
                item.priority, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierTemplatePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierTemplateRule for VerifierTemplatePriorityRangeRule {
    fn check(&self, item: &VerifierTemplate) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierTemplateAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl VerifierTemplateRule for VerifierTemplateAmountRangeRule {
    fn check(&self, item: &VerifierTemplate) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierTemplateCodeLengthRule {
    pub max_length: i64,
}

impl VerifierTemplateRule for VerifierTemplateCodeLengthRule {
    fn check(&self, item: &VerifierTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierTemplateTitleLengthRule {
    pub max_length: i64,
}

impl VerifierTemplateRule for VerifierTemplateTitleLengthRule {
    fn check(&self, item: &VerifierTemplate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier template.
#[derive(Debug, Clone)]
pub struct VerifierTemplateValidator {
    pub rules: Vec<Rc<dyn VerifierTemplateRule>>,
}

impl VerifierTemplateValidator {
    pub fn validate(&self, item: &VerifierTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &VerifierTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_template_validator() -> VerifierTemplateValidator {
    VerifierTemplateValidator {
        rules: vec![
            Rc::new(VerifierTemplateRequiredCodeRule {
                message: "verifier template needs a value".to_string(),
            }) as Rc<dyn VerifierTemplateRule>,
            Rc::new(VerifierTemplatePinnedGuardRule { limit: 120 }),
            Rc::new(VerifierTemplatePriorityRangeRule { min: 32, max: 80 }),
            Rc::new(VerifierTemplateAmountRangeRule { min: 2.2, max: 1886.6 }),
            Rc::new(VerifierTemplateCodeLengthRule { max_length: 32 }),
            Rc::new(VerifierTemplateTitleLengthRule { max_length: 3600 }),
        ],
    }
}
