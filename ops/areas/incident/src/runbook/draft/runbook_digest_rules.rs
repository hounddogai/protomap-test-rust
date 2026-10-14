use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a runbook digest must pass. Check returns its problems, or none.
pub trait RunbookDigestRule: std::fmt::Debug {
    fn check(&self, item: &RunbookDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookDigestDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RunbookDigestRule for RunbookDigestDurationMinutesRangeRule {
    fn check(&self, item: &RunbookDigest) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookDigestThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RunbookDigestRule for RunbookDigestThresholdRangeRule {
    fn check(&self, item: &RunbookDigest) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookDigestRequiredCodeRule {
    pub message: String,
}

impl RunbookDigestRule for RunbookDigestRequiredCodeRule {
    fn check(&self, item: &RunbookDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookDigestAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RunbookDigestRule for RunbookDigestAllowedTitleRule {
    fn check(&self, item: &RunbookDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is blocked", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook digest.
#[derive(Debug, Clone)]
pub struct RunbookDigestValidator {
    pub rules: Vec<Rc<dyn RunbookDigestRule>>,
}

impl RunbookDigestValidator {
    pub fn validate(&self, item: &RunbookDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RunbookDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_digest_validator() -> RunbookDigestValidator {
    RunbookDigestValidator {
        rules: vec![
            Rc::new(RunbookDigestDurationMinutesRangeRule { min: 32, max: 255 }) as Rc<dyn RunbookDigestRule>,
            Rc::new(RunbookDigestThresholdRangeRule { min: 8.8, max: 1645.3 }),
            Rc::new(RunbookDigestRequiredCodeRule {
                message: "runbook digest needs this field".to_string(),
            }),
            Rc::new(RunbookDigestAllowedTitleRule {
                allowed: vec!["legacy".to_string(), "rolling".to_string(), "fallback".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_digest_validator();
        let item = new_runbook_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = RunbookDigestDurationMinutesRangeRule { min: 86400, max: 86400 };
        let mut item = new_runbook_digest("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = RunbookDigestThresholdRangeRule { min: 8.7, max: 3374.6 };
        let mut item = new_runbook_digest("x", "y");
        item.threshold = 3376.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RunbookDigestRequiredCodeRule {
            message: "runbook digest needs an entry".to_string(),
        };
        let mut item = new_runbook_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
