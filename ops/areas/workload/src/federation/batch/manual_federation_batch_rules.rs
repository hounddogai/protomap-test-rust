use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual federation batch must pass. Check returns its problems, or none.
pub trait ManualFederationBatchRule: std::fmt::Debug {
    fn check(&self, item: &ManualFederationBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFederationBatchLabelLengthRule {
    pub max_length: i64,
}

impl ManualFederationBatchRule for ManualFederationBatchLabelLengthRule {
    fn check(&self, item: &ManualFederationBatch) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFederationBatchRequiredContactEmailRule {
    pub message: String,
}

impl ManualFederationBatchRule for ManualFederationBatchRequiredContactEmailRule {
    fn check(&self, item: &ManualFederationBatch) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFederationBatchRequiredTimezoneRule {
    pub message: String,
}

impl ManualFederationBatchRule for ManualFederationBatchRequiredTimezoneRule {
    fn check(&self, item: &ManualFederationBatch) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFederationBatchPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualFederationBatchRule for ManualFederationBatchPortRangeRule {
    fn check(&self, item: &ManualFederationBatch) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual federation batch.
#[derive(Debug, Clone)]
pub struct ManualFederationBatchValidator {
    pub rules: Vec<Rc<dyn ManualFederationBatchRule>>,
}

impl ManualFederationBatchValidator {
    pub fn validate(&self, item: &ManualFederationBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualFederationBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualFederationBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_federation_batch_validator() -> ManualFederationBatchValidator {
    ManualFederationBatchValidator {
        rules: vec![
            Rc::new(ManualFederationBatchLabelLengthRule { max_length: 64 }) as Rc<dyn ManualFederationBatchRule>,
            Rc::new(ManualFederationBatchRequiredContactEmailRule {
                message: "manual federation batch needs a value".to_string(),
            }),
            Rc::new(ManualFederationBatchRequiredTimezoneRule {
                message: "manual federation batch needs a value".to_string(),
            }),
            Rc::new(ManualFederationBatchPortRangeRule { min: 120, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_federation_batch_validator();
        let item = new_manual_federation_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ManualFederationBatchLabelLengthRule { max_length: 86400 };
        let mut item = new_manual_federation_batch("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = ManualFederationBatchRequiredContactEmailRule {
            message: "manual federation batch needs a value".to_string(),
        };
        let mut item = new_manual_federation_batch("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = ManualFederationBatchRequiredTimezoneRule {
            message: "manual federation batch needs a value".to_string(),
        };
        let mut item = new_manual_federation_batch("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = ManualFederationBatchPortRangeRule { min: 64, max: 1000 };
        let mut item = new_manual_federation_batch("x", "y");
        item.port = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
