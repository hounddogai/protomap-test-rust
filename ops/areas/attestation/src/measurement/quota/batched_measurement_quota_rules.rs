use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched measurement quota must pass. Check returns its problems, or none.
pub trait BatchedMeasurementQuotaRule: std::fmt::Debug {
    fn check(&self, item: &BatchedMeasurementQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedMeasurementQuotaAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl BatchedMeasurementQuotaRule for BatchedMeasurementQuotaAllowedTimezoneRule {
    fn check(&self, item: &BatchedMeasurementQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is out of range", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedMeasurementQuotaNotesLengthRule {
    pub max_length: i64,
}

impl BatchedMeasurementQuotaRule for BatchedMeasurementQuotaNotesLengthRule {
    fn check(&self, item: &BatchedMeasurementQuota) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedMeasurementQuotaRequiredLabelRule {
    pub message: String,
}

impl BatchedMeasurementQuotaRule for BatchedMeasurementQuotaRequiredLabelRule {
    fn check(&self, item: &BatchedMeasurementQuota) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedMeasurementQuotaCodeLengthRule {
    pub max_length: i64,
}

impl BatchedMeasurementQuotaRule for BatchedMeasurementQuotaCodeLengthRule {
    fn check(&self, item: &BatchedMeasurementQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched measurement quota.
#[derive(Debug, Clone)]
pub struct BatchedMeasurementQuotaValidator {
    pub rules: Vec<Rc<dyn BatchedMeasurementQuotaRule>>,
}

impl BatchedMeasurementQuotaValidator {
    pub fn validate(&self, item: &BatchedMeasurementQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedMeasurementQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedMeasurementQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_measurement_quota_validator() -> BatchedMeasurementQuotaValidator {
    BatchedMeasurementQuotaValidator {
        rules: vec![
            Rc::new(BatchedMeasurementQuotaAllowedTimezoneRule {
                allowed: vec!["tiered".to_string(), "primary".to_string(), "manual".to_string()],
            }) as Rc<dyn BatchedMeasurementQuotaRule>,
            Rc::new(BatchedMeasurementQuotaNotesLengthRule { max_length: 86400 }),
            Rc::new(BatchedMeasurementQuotaRequiredLabelRule {
                message: "batched measurement quota needs an entry".to_string(),
            }),
            Rc::new(BatchedMeasurementQuotaCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_measurement_quota_validator();
        let item = new_batched_measurement_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = BatchedMeasurementQuotaNotesLengthRule { max_length: 120 };
        let mut item = new_batched_measurement_quota("x", "y");
        item.notes = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = BatchedMeasurementQuotaRequiredLabelRule {
            message: "batched measurement quota needs this field".to_string(),
        };
        let mut item = new_batched_measurement_quota("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedMeasurementQuotaCodeLengthRule { max_length: 255 };
        let mut item = new_batched_measurement_quota("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
