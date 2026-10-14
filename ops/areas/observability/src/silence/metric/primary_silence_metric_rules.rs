use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary silence metric must pass. Check returns its problems, or none.
pub trait PrimarySilenceMetricRule: std::fmt::Debug {
    fn check(&self, item: &PrimarySilenceMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySilenceMetricRequiredExternalRefRule {
    pub message: String,
}

impl PrimarySilenceMetricRule for PrimarySilenceMetricRequiredExternalRefRule {
    fn check(&self, item: &PrimarySilenceMetric) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySilenceMetricChannelLengthRule {
    pub max_length: i64,
}

impl PrimarySilenceMetricRule for PrimarySilenceMetricChannelLengthRule {
    fn check(&self, item: &PrimarySilenceMetric) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySilenceMetricExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimarySilenceMetricRule for PrimarySilenceMetricExpiresAtRangeRule {
    fn check(&self, item: &PrimarySilenceMetric) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySilenceMetricAllowedChannelRule {
    pub allowed: Vec<String>,
}

impl PrimarySilenceMetricRule for PrimarySilenceMetricAllowedChannelRule {
    fn check(&self, item: &PrimarySilenceMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.channel)))
        {
            return vec![format!("channel {} is expired", item.channel)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary silence metric.
#[derive(Debug, Clone)]
pub struct PrimarySilenceMetricValidator {
    pub rules: Vec<Rc<dyn PrimarySilenceMetricRule>>,
}

impl PrimarySilenceMetricValidator {
    pub fn validate(&self, item: &PrimarySilenceMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimarySilenceMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimarySilenceMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_silence_metric_validator() -> PrimarySilenceMetricValidator {
    PrimarySilenceMetricValidator {
        rules: vec![
            Rc::new(PrimarySilenceMetricRequiredExternalRefRule {
                message: "primary silence metric needs this field".to_string(),
            }) as Rc<dyn PrimarySilenceMetricRule>,
            Rc::new(PrimarySilenceMetricChannelLengthRule { max_length: 86400 }),
            Rc::new(PrimarySilenceMetricExpiresAtRangeRule { min: 64, max: 255 }),
            Rc::new(PrimarySilenceMetricAllowedChannelRule {
                allowed: vec!["manual".to_string(), "capped".to_string(), "default".to_string()],
            }),
        ],
    }
}
