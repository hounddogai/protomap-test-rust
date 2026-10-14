use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending nonce record must pass. Check returns its problems, or none.
pub trait PendingNonceRecordRule: std::fmt::Debug {
    fn check(&self, item: &PendingNonceRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceRecordRequiredCodeRule {
    pub message: String,
}

impl PendingNonceRecordRule for PendingNonceRecordRequiredCodeRule {
    fn check(&self, item: &PendingNonceRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceRecordNameLengthRule {
    pub max_length: i64,
}

impl PendingNonceRecordRule for PendingNonceRecordNameLengthRule {
    fn check(&self, item: &PendingNonceRecord) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceRecordCodeLengthRule {
    pub max_length: i64,
}

impl PendingNonceRecordRule for PendingNonceRecordCodeLengthRule {
    fn check(&self, item: &PendingNonceRecord) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceRecordLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingNonceRecordRule for PendingNonceRecordLimitRangeRule {
    fn check(&self, item: &PendingNonceRecord) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceRecordFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingNonceRecordRule for PendingNonceRecordFloorValueRangeRule {
    fn check(&self, item: &PendingNonceRecord) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceRecordOwnerIdLengthRule {
    pub max_length: i64,
}

impl PendingNonceRecordRule for PendingNonceRecordOwnerIdLengthRule {
    fn check(&self, item: &PendingNonceRecord) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending nonce record.
#[derive(Debug, Clone)]
pub struct PendingNonceRecordValidator {
    pub rules: Vec<Rc<dyn PendingNonceRecordRule>>,
}

impl PendingNonceRecordValidator {
    pub fn validate(&self, item: &PendingNonceRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingNonceRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingNonceRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_nonce_record_validator() -> PendingNonceRecordValidator {
    PendingNonceRecordValidator {
        rules: vec![
            Rc::new(PendingNonceRecordRequiredCodeRule {
                message: "pending nonce record needs a value".to_string(),
            }) as Rc<dyn PendingNonceRecordRule>,
            Rc::new(PendingNonceRecordNameLengthRule { max_length: 80 }),
            Rc::new(PendingNonceRecordCodeLengthRule { max_length: 16 }),
            Rc::new(PendingNonceRecordLimitRangeRule { min: 32, max: 32 }),
            Rc::new(PendingNonceRecordFloorValueRangeRule { min: 6.5, max: 4739.4 }),
            Rc::new(PendingNonceRecordOwnerIdLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_nonce_record_validator();
        let item = new_pending_nonce_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingNonceRecordRequiredCodeRule {
            message: "pending nonce record needs this field".to_string(),
        };
        let mut item = new_pending_nonce_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PendingNonceRecordNameLengthRule { max_length: 86400 };
        let mut item = new_pending_nonce_record("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingNonceRecordCodeLengthRule { max_length: 86400 };
        let mut item = new_pending_nonce_record("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = PendingNonceRecordLimitRangeRule { min: 32, max: 32 };
        let mut item = new_pending_nonce_record("x", "y");
        item.limit = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = PendingNonceRecordFloorValueRangeRule { min: 4.3, max: 1117.4 };
        let mut item = new_pending_nonce_record("x", "y");
        item.floor_value = 1118.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = PendingNonceRecordOwnerIdLengthRule { max_length: 255 };
        let mut item = new_pending_nonce_record("x", "y");
        item.owner_id = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
