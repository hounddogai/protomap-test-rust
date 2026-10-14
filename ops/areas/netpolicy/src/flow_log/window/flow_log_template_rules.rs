use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a flow log template must pass. Check returns its problems, or none.
pub trait FlowLogTemplateRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogTemplateCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FlowLogTemplateRule for FlowLogTemplateCostRangeRule {
    fn check(&self, item: &FlowLogTemplate) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogTemplateRequiredCodeRule {
    pub message: String,
}

impl FlowLogTemplateRule for FlowLogTemplateRequiredCodeRule {
    fn check(&self, item: &FlowLogTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogTemplateRequiredSourceRule {
    pub message: String,
}

impl FlowLogTemplateRule for FlowLogTemplateRequiredSourceRule {
    fn check(&self, item: &FlowLogTemplate) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogTemplateRequiredTimezoneRule {
    pub message: String,
}

impl FlowLogTemplateRule for FlowLogTemplateRequiredTimezoneRule {
    fn check(&self, item: &FlowLogTemplate) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogTemplateTitleLengthRule {
    pub max_length: i64,
}

impl FlowLogTemplateRule for FlowLogTemplateTitleLengthRule {
    fn check(&self, item: &FlowLogTemplate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogTemplateRequiredTitleRule {
    pub message: String,
}

impl FlowLogTemplateRule for FlowLogTemplateRequiredTitleRule {
    fn check(&self, item: &FlowLogTemplate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log template.
#[derive(Debug, Clone)]
pub struct FlowLogTemplateValidator {
    pub rules: Vec<Rc<dyn FlowLogTemplateRule>>,
}

impl FlowLogTemplateValidator {
    pub fn validate(&self, item: &FlowLogTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FlowLogTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_template_validator() -> FlowLogTemplateValidator {
    FlowLogTemplateValidator {
        rules: vec![
            Rc::new(FlowLogTemplateCostRangeRule { min: 7.0, max: 560.9 }) as Rc<dyn FlowLogTemplateRule>,
            Rc::new(FlowLogTemplateRequiredCodeRule {
                message: "flow log template needs this field".to_string(),
            }),
            Rc::new(FlowLogTemplateRequiredSourceRule {
                message: "flow log template needs this field".to_string(),
            }),
            Rc::new(FlowLogTemplateRequiredTimezoneRule {
                message: "flow log template needs this field".to_string(),
            }),
            Rc::new(FlowLogTemplateTitleLengthRule { max_length: 80 }),
            Rc::new(FlowLogTemplateRequiredTitleRule {
                message: "flow log template needs an entry".to_string(),
            }),
        ],
    }
}
