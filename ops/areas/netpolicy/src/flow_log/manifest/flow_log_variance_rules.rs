use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a flow log variance must pass. Check returns its problems, or none.
pub trait FlowLogVarianceRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogVarianceAudienceLengthRule {
    pub max_length: i64,
}

impl FlowLogVarianceRule for FlowLogVarianceAudienceLengthRule {
    fn check(&self, item: &FlowLogVariance) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogVarianceCodeLengthRule {
    pub max_length: i64,
}

impl FlowLogVarianceRule for FlowLogVarianceCodeLengthRule {
    fn check(&self, item: &FlowLogVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogVarianceRequiredNameRule {
    pub message: String,
}

impl FlowLogVarianceRule for FlowLogVarianceRequiredNameRule {
    fn check(&self, item: &FlowLogVariance) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogVarianceRequiredAudienceRule {
    pub message: String,
}

impl FlowLogVarianceRule for FlowLogVarianceRequiredAudienceRule {
    fn check(&self, item: &FlowLogVariance) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogVarianceAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl FlowLogVarianceRule for FlowLogVarianceAllowedAudienceRule {
    fn check(&self, item: &FlowLogVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is above maximum", item.audience)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogVarianceBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FlowLogVarianceRule for FlowLogVarianceBurstRangeRule {
    fn check(&self, item: &FlowLogVariance) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a flow log variance.
#[derive(Debug, Clone)]
pub struct FlowLogVarianceValidator {
    pub rules: Vec<Rc<dyn FlowLogVarianceRule>>,
}

impl FlowLogVarianceValidator {
    pub fn validate(&self, item: &FlowLogVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FlowLogVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_variance_validator() -> FlowLogVarianceValidator {
    FlowLogVarianceValidator {
        rules: vec![
            Rc::new(FlowLogVarianceAudienceLengthRule { max_length: 16 }) as Rc<dyn FlowLogVarianceRule>,
            Rc::new(FlowLogVarianceCodeLengthRule { max_length: 255 }),
            Rc::new(FlowLogVarianceRequiredNameRule {
                message: "flow log variance needs an entry".to_string(),
            }),
            Rc::new(FlowLogVarianceRequiredAudienceRule {
                message: "flow log variance needs a value".to_string(),
            }),
            Rc::new(FlowLogVarianceAllowedAudienceRule {
                allowed: vec!["seasonal".to_string()],
            }),
            Rc::new(FlowLogVarianceBurstRangeRule { min: 80, max: 255 }),
        ],
    }
}
