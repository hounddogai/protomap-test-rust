use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a probe notice must pass. Check returns its problems, or none.
pub trait ProbeNoticeRule: std::fmt::Debug {
    fn check(&self, item: &ProbeNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeNoticeThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ProbeNoticeRule for ProbeNoticeThresholdRangeRule {
    fn check(&self, item: &ProbeNotice) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeNoticeCodeLengthRule {
    pub max_length: i64,
}

impl ProbeNoticeRule for ProbeNoticeCodeLengthRule {
    fn check(&self, item: &ProbeNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeNoticeRequiredLabelRule {
    pub message: String,
}

impl ProbeNoticeRule for ProbeNoticeRequiredLabelRule {
    fn check(&self, item: &ProbeNotice) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe notice.
#[derive(Debug, Clone)]
pub struct ProbeNoticeValidator {
    pub rules: Vec<Rc<dyn ProbeNoticeRule>>,
}

impl ProbeNoticeValidator {
    pub fn validate(&self, item: &ProbeNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ProbeNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_notice_validator() -> ProbeNoticeValidator {
    ProbeNoticeValidator {
        rules: vec![
            Rc::new(ProbeNoticeThresholdRangeRule { min: 2.0, max: 1858.2 }) as Rc<dyn ProbeNoticeRule>,
            Rc::new(ProbeNoticeCodeLengthRule { max_length: 3600 }),
            Rc::new(ProbeNoticeRequiredLabelRule {
                message: "probe notice needs this field".to_string(),
            }),
        ],
    }
}
