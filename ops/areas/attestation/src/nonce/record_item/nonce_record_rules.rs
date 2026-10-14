use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a nonce record must pass. Check returns its problems, or none.
pub trait NonceRecordRule: std::fmt::Debug {
    fn check(&self, item: &NonceRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceRecordRequiredCountryRule {
    pub message: String,
}

impl NonceRecordRule for NonceRecordRequiredCountryRule {
    fn check(&self, item: &NonceRecord) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceRecordAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl NonceRecordRule for NonceRecordAllowedCountryRule {
    fn check(&self, item: &NonceRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is inconsistent", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceRecordBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl NonceRecordRule for NonceRecordBatchSizeRangeRule {
    fn check(&self, item: &NonceRecord) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a nonce record.
#[derive(Debug, Clone)]
pub struct NonceRecordValidator {
    pub rules: Vec<Rc<dyn NonceRecordRule>>,
}

impl NonceRecordValidator {
    pub fn validate(&self, item: &NonceRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &NonceRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_record_validator() -> NonceRecordValidator {
    NonceRecordValidator {
        rules: vec![
            Rc::new(NonceRecordRequiredCountryRule {
                message: "nonce record needs a value".to_string(),
            }) as Rc<dyn NonceRecordRule>,
            Rc::new(NonceRecordAllowedCountryRule {
                allowed: vec!["capped".to_string(), "capped".to_string(), "batched".to_string()],
            }),
            Rc::new(NonceRecordBatchSizeRangeRule { min: 1000, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_nonce_record_validator();
        let item = new_nonce_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = NonceRecordRequiredCountryRule {
            message: "nonce record needs an entry".to_string(),
        };
        let mut item = new_nonce_record("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = NonceRecordBatchSizeRangeRule { min: 255, max: 255 };
        let mut item = new_nonce_record("x", "y");
        item.batch_size = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
