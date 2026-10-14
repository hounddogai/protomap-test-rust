use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary trust domain template must pass. Check returns its problems, or none.
pub trait PrimaryTrustDomainTemplateRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryTrustDomainTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTrustDomainTemplateBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryTrustDomainTemplateRule for PrimaryTrustDomainTemplateBatchSizeRangeRule {
    fn check(&self, item: &PrimaryTrustDomainTemplate) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTrustDomainTemplateReviewerLengthRule {
    pub max_length: i64,
}

impl PrimaryTrustDomainTemplateRule for PrimaryTrustDomainTemplateReviewerLengthRule {
    fn check(&self, item: &PrimaryTrustDomainTemplate) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTrustDomainTemplateRequiredReviewerRule {
    pub message: String,
}

impl PrimaryTrustDomainTemplateRule for PrimaryTrustDomainTemplateRequiredReviewerRule {
    fn check(&self, item: &PrimaryTrustDomainTemplate) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary trust domain template.
#[derive(Debug, Clone)]
pub struct PrimaryTrustDomainTemplateValidator {
    pub rules: Vec<Rc<dyn PrimaryTrustDomainTemplateRule>>,
}

impl PrimaryTrustDomainTemplateValidator {
    pub fn validate(&self, item: &PrimaryTrustDomainTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryTrustDomainTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryTrustDomainTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_trust_domain_template_validator() -> PrimaryTrustDomainTemplateValidator {
    PrimaryTrustDomainTemplateValidator {
        rules: vec![
            Rc::new(PrimaryTrustDomainTemplateBatchSizeRangeRule { min: 255, max: 1000 })
                as Rc<dyn PrimaryTrustDomainTemplateRule>,
            Rc::new(PrimaryTrustDomainTemplateReviewerLengthRule { max_length: 1000 }),
            Rc::new(PrimaryTrustDomainTemplateRequiredReviewerRule {
                message: "primary trust domain template needs this field".to_string(),
            }),
        ],
    }
}
