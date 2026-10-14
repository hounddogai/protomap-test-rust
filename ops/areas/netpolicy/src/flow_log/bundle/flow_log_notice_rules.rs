use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a flow log notice must pass. Check returns its problems, or none.
pub trait FlowLogNoticeRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogNoticeTitleLengthRule {
    pub max_length: i64,
}

impl FlowLogNoticeRule for FlowLogNoticeTitleLengthRule {
    fn check(&self, item: &FlowLogNotice) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogNoticeRequiredTitleRule {
    pub message: String,
}

impl FlowLogNoticeRule for FlowLogNoticeRequiredTitleRule {
    fn check(&self, item: &FlowLogNotice) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogNoticeAllowedCurrencyRule {
    pub allowed: Vec<String>,
}

impl FlowLogNoticeRule for FlowLogNoticeAllowedCurrencyRule {
    fn check(&self, item: &FlowLogNotice) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.currency)))
        {
            return vec![format!("currency {} is stale", item.currency)];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log notice.
#[derive(Debug, Clone)]
pub struct FlowLogNoticeValidator {
    pub rules: Vec<Rc<dyn FlowLogNoticeRule>>,
}

impl FlowLogNoticeValidator {
    pub fn validate(&self, item: &FlowLogNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FlowLogNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_notice_validator() -> FlowLogNoticeValidator {
    FlowLogNoticeValidator {
        rules: vec![
            Rc::new(FlowLogNoticeTitleLengthRule { max_length: 80 }) as Rc<dyn FlowLogNoticeRule>,
            Rc::new(FlowLogNoticeRequiredTitleRule {
                message: "flow log notice needs this field".to_string(),
            }),
            Rc::new(FlowLogNoticeAllowedCurrencyRule { allowed: Vec::new() }),
        ],
    }
}
