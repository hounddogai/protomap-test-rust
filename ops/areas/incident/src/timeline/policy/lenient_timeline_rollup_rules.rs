use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient timeline rollup must pass. Check returns its problems, or none.
pub trait LenientTimelineRollupRule: std::fmt::Debug {
    fn check(&self, item: &LenientTimelineRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTimelineRollupLabelLengthRule {
    pub max_length: i64,
}

impl LenientTimelineRollupRule for LenientTimelineRollupLabelLengthRule {
    fn check(&self, item: &LenientTimelineRollup) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTimelineRollupRequiredPrincipalRule {
    pub message: String,
}

impl LenientTimelineRollupRule for LenientTimelineRollupRequiredPrincipalRule {
    fn check(&self, item: &LenientTimelineRollup) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTimelineRollupPrincipalLengthRule {
    pub max_length: i64,
}

impl LenientTimelineRollupRule for LenientTimelineRollupPrincipalLengthRule {
    fn check(&self, item: &LenientTimelineRollup) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient timeline rollup.
#[derive(Debug, Clone)]
pub struct LenientTimelineRollupValidator {
    pub rules: Vec<Rc<dyn LenientTimelineRollupRule>>,
}

impl LenientTimelineRollupValidator {
    pub fn validate(&self, item: &LenientTimelineRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientTimelineRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientTimelineRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_timeline_rollup_validator() -> LenientTimelineRollupValidator {
    LenientTimelineRollupValidator {
        rules: vec![
            Rc::new(LenientTimelineRollupLabelLengthRule { max_length: 1000 }) as Rc<dyn LenientTimelineRollupRule>,
            Rc::new(LenientTimelineRollupRequiredPrincipalRule {
                message: "lenient timeline rollup needs an entry".to_string(),
            }),
            Rc::new(LenientTimelineRollupPrincipalLengthRule { max_length: 86400 }),
        ],
    }
}
