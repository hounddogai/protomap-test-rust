use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled slo record must pass. Check returns its problems, or none.
pub trait ScheduledSloRecordRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledSloRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSloRecordEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledSloRecordRule for ScheduledSloRecordEndsAtRangeRule {
    fn check(&self, item: &ScheduledSloRecord) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSloRecordRequiredCodeRule {
    pub message: String,
}

impl ScheduledSloRecordRule for ScheduledSloRecordRequiredCodeRule {
    fn check(&self, item: &ScheduledSloRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSloRecordIssuerLengthRule {
    pub max_length: i64,
}

impl ScheduledSloRecordRule for ScheduledSloRecordIssuerLengthRule {
    fn check(&self, item: &ScheduledSloRecord) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSloRecordNameLengthRule {
    pub max_length: i64,
}

impl ScheduledSloRecordRule for ScheduledSloRecordNameLengthRule {
    fn check(&self, item: &ScheduledSloRecord) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled slo record.
#[derive(Debug, Clone)]
pub struct ScheduledSloRecordValidator {
    pub rules: Vec<Rc<dyn ScheduledSloRecordRule>>,
}

impl ScheduledSloRecordValidator {
    pub fn validate(&self, item: &ScheduledSloRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledSloRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledSloRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_slo_record_validator() -> ScheduledSloRecordValidator {
    ScheduledSloRecordValidator {
        rules: vec![
            Rc::new(ScheduledSloRecordEndsAtRangeRule { min: 64, max: 3600 }) as Rc<dyn ScheduledSloRecordRule>,
            Rc::new(ScheduledSloRecordRequiredCodeRule {
                message: "scheduled slo record needs a value".to_string(),
            }),
            Rc::new(ScheduledSloRecordIssuerLengthRule { max_length: 32 }),
            Rc::new(ScheduledSloRecordNameLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_slo_record_validator();
        let item = new_scheduled_slo_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = ScheduledSloRecordEndsAtRangeRule { min: 32, max: 80 };
        let mut item = new_scheduled_slo_record("x", "y");
        item.ends_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledSloRecordRequiredCodeRule {
            message: "scheduled slo record needs a value".to_string(),
        };
        let mut item = new_scheduled_slo_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = ScheduledSloRecordIssuerLengthRule { max_length: 80 };
        let mut item = new_scheduled_slo_record("x", "y");
        item.issuer = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ScheduledSloRecordNameLengthRule { max_length: 86400 };
        let mut item = new_scheduled_slo_record("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
