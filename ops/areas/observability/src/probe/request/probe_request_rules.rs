use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a probe request must pass. Check returns its problems, or none.
pub trait ProbeRequestRule: std::fmt::Debug {
    fn check(&self, item: &ProbeRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeRequestRequiredLabelRule {
    pub message: String,
}

impl ProbeRequestRule for ProbeRequestRequiredLabelRule {
    fn check(&self, item: &ProbeRequest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeRequestScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ProbeRequestRule for ProbeRequestScoreRangeRule {
    fn check(&self, item: &ProbeRequest) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeRequestCodePrefixRule {
    pub prefix: String,
}

impl ProbeRequestRule for ProbeRequestCodePrefixRule {
    fn check(&self, item: &ProbeRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe request.
#[derive(Debug, Clone)]
pub struct ProbeRequestValidator {
    pub rules: Vec<Rc<dyn ProbeRequestRule>>,
}

impl ProbeRequestValidator {
    pub fn validate(&self, item: &ProbeRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ProbeRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_request_validator() -> ProbeRequestValidator {
    ProbeRequestValidator {
        rules: vec![
            Rc::new(ProbeRequestRequiredLabelRule {
                message: "probe request needs this field".to_string(),
            }) as Rc<dyn ProbeRequestRule>,
            Rc::new(ProbeRequestScoreRangeRule { min: 9.2, max: 4505.1 }),
            Rc::new(ProbeRequestCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
