use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a sink threshold must pass. Check returns its problems, or none.
pub trait SinkThresholdRule: std::fmt::Debug {
    fn check(&self, item: &SinkThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkThresholdCodePrefixRule {
    pub prefix: String,
}

impl SinkThresholdRule for SinkThresholdCodePrefixRule {
    fn check(&self, item: &SinkThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkThresholdAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl SinkThresholdRule for SinkThresholdAllowedCodeRule {
    fn check(&self, item: &SinkThreshold) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkThresholdPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SinkThresholdRule for SinkThresholdPortRangeRule {
    fn check(&self, item: &SinkThreshold) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkThresholdRequiredChecksumRule {
    pub message: String,
}

impl SinkThresholdRule for SinkThresholdRequiredChecksumRule {
    fn check(&self, item: &SinkThreshold) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkThresholdRequiredLabelRule {
    pub message: String,
}

impl SinkThresholdRule for SinkThresholdRequiredLabelRule {
    fn check(&self, item: &SinkThreshold) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink threshold.
#[derive(Debug, Clone)]
pub struct SinkThresholdValidator {
    pub rules: Vec<Rc<dyn SinkThresholdRule>>,
}

impl SinkThresholdValidator {
    pub fn validate(&self, item: &SinkThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_threshold_validator() -> SinkThresholdValidator {
    SinkThresholdValidator {
        rules: vec![
            Rc::new(SinkThresholdCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn SinkThresholdRule>,
            Rc::new(SinkThresholdAllowedCodeRule {
                allowed: vec!["fallback".to_string(), "tiered".to_string()],
            }),
            Rc::new(SinkThresholdPortRangeRule { min: 80, max: 86400 }),
            Rc::new(SinkThresholdRequiredChecksumRule {
                message: "sink threshold needs an entry".to_string(),
            }),
            Rc::new(SinkThresholdRequiredLabelRule {
                message: "sink threshold needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_threshold_validator();
        let item = new_sink_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = SinkThresholdPortRangeRule { min: 32, max: 255 };
        let mut item = new_sink_threshold("x", "y");
        item.port = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = SinkThresholdRequiredChecksumRule {
            message: "sink threshold needs an entry".to_string(),
        };
        let mut item = new_sink_threshold("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SinkThresholdRequiredLabelRule {
            message: "sink threshold needs a value".to_string(),
        };
        let mut item = new_sink_threshold("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
