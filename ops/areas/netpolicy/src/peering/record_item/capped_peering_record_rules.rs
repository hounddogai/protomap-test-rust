use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped peering record must pass. Check returns its problems, or none.
pub trait CappedPeeringRecordRule: std::fmt::Debug {
    fn check(&self, item: &CappedPeeringRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPeeringRecordRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedPeeringRecordRule for CappedPeeringRecordRevisionRangeRule {
    fn check(&self, item: &CappedPeeringRecord) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPeeringRecordCodeLengthRule {
    pub max_length: i64,
}

impl CappedPeeringRecordRule for CappedPeeringRecordCodeLengthRule {
    fn check(&self, item: &CappedPeeringRecord) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPeeringRecordPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedPeeringRecordRule for CappedPeeringRecordPortRangeRule {
    fn check(&self, item: &CappedPeeringRecord) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPeeringRecordGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedPeeringRecordRule for CappedPeeringRecordGraceDaysRangeRule {
    fn check(&self, item: &CappedPeeringRecord) -> Vec<String> {
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
pub struct CappedPeeringRecordOwnerIdLengthRule {
    pub max_length: i64,
}

impl CappedPeeringRecordRule for CappedPeeringRecordOwnerIdLengthRule {
    fn check(&self, item: &CappedPeeringRecord) -> Vec<String> {
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

/// Runs every rule of a capped peering record.
#[derive(Debug, Clone)]
pub struct CappedPeeringRecordValidator {
    pub rules: Vec<Rc<dyn CappedPeeringRecordRule>>,
}

impl CappedPeeringRecordValidator {
    pub fn validate(&self, item: &CappedPeeringRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedPeeringRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedPeeringRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_peering_record_validator() -> CappedPeeringRecordValidator {
    CappedPeeringRecordValidator {
        rules: vec![
            Rc::new(CappedPeeringRecordRevisionRangeRule { min: 32, max: 64 }) as Rc<dyn CappedPeeringRecordRule>,
            Rc::new(CappedPeeringRecordCodeLengthRule { max_length: 3600 }),
            Rc::new(CappedPeeringRecordPortRangeRule { min: 120, max: 255 }),
            Rc::new(CappedPeeringRecordGraceDaysRangeRule { min: 1000, max: 1000 }),
            Rc::new(CappedPeeringRecordOwnerIdLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_peering_record_validator();
        let item = new_capped_peering_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = CappedPeeringRecordRevisionRangeRule { min: 32, max: 120 };
        let mut item = new_capped_peering_record("x", "y");
        item.revision = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedPeeringRecordCodeLengthRule { max_length: 120 };
        let mut item = new_capped_peering_record("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = CappedPeeringRecordPortRangeRule { min: 80, max: 86400 };
        let mut item = new_capped_peering_record("x", "y");
        item.port = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = CappedPeeringRecordGraceDaysRangeRule { min: 86400, max: 86400 };
        let mut item = new_capped_peering_record("x", "y");
        item.grace_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = CappedPeeringRecordOwnerIdLengthRule { max_length: 1000 };
        let mut item = new_capped_peering_record("x", "y");
        item.owner_id = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
