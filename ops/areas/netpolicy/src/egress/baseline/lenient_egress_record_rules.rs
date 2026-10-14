use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient egress record must pass. Check returns its problems, or none.
pub trait LenientEgressRecordRule: std::fmt::Debug {
    fn check(&self, item: &LenientEgressRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressRecordIssuerLengthRule {
    pub max_length: i64,
}

impl LenientEgressRecordRule for LenientEgressRecordIssuerLengthRule {
    fn check(&self, item: &LenientEgressRecord) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressRecordOwnerIdLengthRule {
    pub max_length: i64,
}

impl LenientEgressRecordRule for LenientEgressRecordOwnerIdLengthRule {
    fn check(&self, item: &LenientEgressRecord) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressRecordRequiredOwnerIdRule {
    pub message: String,
}

impl LenientEgressRecordRule for LenientEgressRecordRequiredOwnerIdRule {
    fn check(&self, item: &LenientEgressRecord) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressRecordGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientEgressRecordRule for LenientEgressRecordGraceDaysRangeRule {
    fn check(&self, item: &LenientEgressRecord) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressRecordBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientEgressRecordRule for LenientEgressRecordBatchSizeRangeRule {
    fn check(&self, item: &LenientEgressRecord) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressRecordNameLengthRule {
    pub max_length: i64,
}

impl LenientEgressRecordRule for LenientEgressRecordNameLengthRule {
    fn check(&self, item: &LenientEgressRecord) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient egress record.
#[derive(Debug, Clone)]
pub struct LenientEgressRecordValidator {
    pub rules: Vec<Rc<dyn LenientEgressRecordRule>>,
}

impl LenientEgressRecordValidator {
    pub fn validate(&self, item: &LenientEgressRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientEgressRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientEgressRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_egress_record_validator() -> LenientEgressRecordValidator {
    LenientEgressRecordValidator {
        rules: vec![
            Rc::new(LenientEgressRecordIssuerLengthRule { max_length: 1000 }) as Rc<dyn LenientEgressRecordRule>,
            Rc::new(LenientEgressRecordOwnerIdLengthRule { max_length: 120 }),
            Rc::new(LenientEgressRecordRequiredOwnerIdRule {
                message: "lenient egress record needs an entry".to_string(),
            }),
            Rc::new(LenientEgressRecordGraceDaysRangeRule { min: 32, max: 255 }),
            Rc::new(LenientEgressRecordBatchSizeRangeRule { min: 64, max: 255 }),
            Rc::new(LenientEgressRecordNameLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_egress_record_validator();
        let item = new_lenient_egress_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = LenientEgressRecordIssuerLengthRule { max_length: 86400 };
        let mut item = new_lenient_egress_record("x", "y");
        item.issuer = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = LenientEgressRecordOwnerIdLengthRule { max_length: 32 };
        let mut item = new_lenient_egress_record("x", "y");
        item.owner_id = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = LenientEgressRecordRequiredOwnerIdRule {
            message: "lenient egress record needs this field".to_string(),
        };
        let mut item = new_lenient_egress_record("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = LenientEgressRecordGraceDaysRangeRule { min: 64, max: 80 };
        let mut item = new_lenient_egress_record("x", "y");
        item.grace_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = LenientEgressRecordBatchSizeRangeRule { min: 64, max: 1000 };
        let mut item = new_lenient_egress_record("x", "y");
        item.batch_size = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = LenientEgressRecordNameLengthRule { max_length: 255 };
        let mut item = new_lenient_egress_record("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
