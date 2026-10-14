use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a query template must pass. Check returns its problems, or none.
pub trait QueryTemplateRule: std::fmt::Debug {
    fn check(&self, item: &QueryTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryTemplateRequiredTitleRule {
    pub message: String,
}

impl QueryTemplateRule for QueryTemplateRequiredTitleRule {
    fn check(&self, item: &QueryTemplate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryTemplateAllowedHostRule {
    pub allowed: Vec<String>,
}

impl QueryTemplateRule for QueryTemplateAllowedHostRule {
    fn check(&self, item: &QueryTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.host))) {
            return vec![format!("host {} is stale", item.host)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryTemplateDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QueryTemplateRule for QueryTemplateDiscountRangeRule {
    fn check(&self, item: &QueryTemplate) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryTemplateTitleLengthRule {
    pub max_length: i64,
}

impl QueryTemplateRule for QueryTemplateTitleLengthRule {
    fn check(&self, item: &QueryTemplate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryTemplateRequiredPhoneRule {
    pub message: String,
}

impl QueryTemplateRule for QueryTemplateRequiredPhoneRule {
    fn check(&self, item: &QueryTemplate) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a query template.
#[derive(Debug, Clone)]
pub struct QueryTemplateValidator {
    pub rules: Vec<Rc<dyn QueryTemplateRule>>,
}

impl QueryTemplateValidator {
    pub fn validate(&self, item: &QueryTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QueryTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_template_validator() -> QueryTemplateValidator {
    QueryTemplateValidator {
        rules: vec![
            Rc::new(QueryTemplateRequiredTitleRule {
                message: "query template needs this field".to_string(),
            }) as Rc<dyn QueryTemplateRule>,
            Rc::new(QueryTemplateAllowedHostRule {
                allowed: vec!["capped".to_string(), "scheduled".to_string()],
            }),
            Rc::new(QueryTemplateDiscountRangeRule { min: 5.4, max: 4614.6 }),
            Rc::new(QueryTemplateTitleLengthRule { max_length: 64 }),
            Rc::new(QueryTemplateRequiredPhoneRule {
                message: "query template needs this field".to_string(),
            }),
        ],
    }
}
