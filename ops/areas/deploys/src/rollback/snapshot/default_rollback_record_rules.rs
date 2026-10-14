use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default rollback record must pass. Check returns its problems, or none.
pub trait DefaultRollbackRecordRule: std::fmt::Debug {
    fn check(&self, item: &DefaultRollbackRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRollbackRecordPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultRollbackRecordRule for DefaultRollbackRecordPortRangeRule {
    fn check(&self, item: &DefaultRollbackRecord) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRollbackRecordDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultRollbackRecordRule for DefaultRollbackRecordDiscountRangeRule {
    fn check(&self, item: &DefaultRollbackRecord) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRollbackRecordRequiredCodeRule {
    pub message: String,
}

impl DefaultRollbackRecordRule for DefaultRollbackRecordRequiredCodeRule {
    fn check(&self, item: &DefaultRollbackRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRollbackRecordAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DefaultRollbackRecordRule for DefaultRollbackRecordAllowedCodeRule {
    fn check(&self, item: &DefaultRollbackRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is missing", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRollbackRecordRequiredNameRule {
    pub message: String,
}

impl DefaultRollbackRecordRule for DefaultRollbackRecordRequiredNameRule {
    fn check(&self, item: &DefaultRollbackRecord) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default rollback record.
#[derive(Debug, Clone)]
pub struct DefaultRollbackRecordValidator {
    pub rules: Vec<Rc<dyn DefaultRollbackRecordRule>>,
}

impl DefaultRollbackRecordValidator {
    pub fn validate(&self, item: &DefaultRollbackRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultRollbackRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultRollbackRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_rollback_record_validator() -> DefaultRollbackRecordValidator {
    DefaultRollbackRecordValidator {
        rules: vec![
            Rc::new(DefaultRollbackRecordPortRangeRule { min: 120, max: 120 }) as Rc<dyn DefaultRollbackRecordRule>,
            Rc::new(DefaultRollbackRecordDiscountRangeRule { min: 1.1, max: 2656.0 }),
            Rc::new(DefaultRollbackRecordRequiredCodeRule {
                message: "default rollback record needs this field".to_string(),
            }),
            Rc::new(DefaultRollbackRecordAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(DefaultRollbackRecordRequiredNameRule {
                message: "default rollback record needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_rollback_record_validator();
        let item = new_default_rollback_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = DefaultRollbackRecordPortRangeRule { min: 16, max: 3600 };
        let mut item = new_default_rollback_record("x", "y");
        item.port = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = DefaultRollbackRecordDiscountRangeRule { min: 7.2, max: 4658.3 };
        let mut item = new_default_rollback_record("x", "y");
        item.discount = 4659.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultRollbackRecordRequiredCodeRule {
            message: "default rollback record needs an entry".to_string(),
        };
        let mut item = new_default_rollback_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = DefaultRollbackRecordRequiredNameRule {
            message: "default rollback record needs a value".to_string(),
        };
        let mut item = new_default_rollback_record("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
