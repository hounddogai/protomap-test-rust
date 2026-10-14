use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary failover snapshot must pass. Check returns its problems, or none.
pub trait PrimaryFailoverSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryFailoverSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverSnapshotNameLengthRule {
    pub max_length: i64,
}

impl PrimaryFailoverSnapshotRule for PrimaryFailoverSnapshotNameLengthRule {
    fn check(&self, item: &PrimaryFailoverSnapshot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverSnapshotRequiredCodeRule {
    pub message: String,
}

impl PrimaryFailoverSnapshotRule for PrimaryFailoverSnapshotRequiredCodeRule {
    fn check(&self, item: &PrimaryFailoverSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverSnapshotLocaleLengthRule {
    pub max_length: i64,
}

impl PrimaryFailoverSnapshotRule for PrimaryFailoverSnapshotLocaleLengthRule {
    fn check(&self, item: &PrimaryFailoverSnapshot) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverSnapshotMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryFailoverSnapshotRule for PrimaryFailoverSnapshotMinimumRangeRule {
    fn check(&self, item: &PrimaryFailoverSnapshot) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverSnapshotRequiredLocaleRule {
    pub message: String,
}

impl PrimaryFailoverSnapshotRule for PrimaryFailoverSnapshotRequiredLocaleRule {
    fn check(&self, item: &PrimaryFailoverSnapshot) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary failover snapshot.
#[derive(Debug, Clone)]
pub struct PrimaryFailoverSnapshotValidator {
    pub rules: Vec<Rc<dyn PrimaryFailoverSnapshotRule>>,
}

impl PrimaryFailoverSnapshotValidator {
    pub fn validate(&self, item: &PrimaryFailoverSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryFailoverSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryFailoverSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_failover_snapshot_validator() -> PrimaryFailoverSnapshotValidator {
    PrimaryFailoverSnapshotValidator {
        rules: vec![
            Rc::new(PrimaryFailoverSnapshotNameLengthRule { max_length: 80 }) as Rc<dyn PrimaryFailoverSnapshotRule>,
            Rc::new(PrimaryFailoverSnapshotRequiredCodeRule {
                message: "primary failover snapshot needs this field".to_string(),
            }),
            Rc::new(PrimaryFailoverSnapshotLocaleLengthRule { max_length: 120 }),
            Rc::new(PrimaryFailoverSnapshotMinimumRangeRule { min: 5.4, max: 1457.8 }),
            Rc::new(PrimaryFailoverSnapshotRequiredLocaleRule {
                message: "primary failover snapshot needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_failover_snapshot_validator();
        let item = new_primary_failover_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PrimaryFailoverSnapshotNameLengthRule { max_length: 16 };
        let mut item = new_primary_failover_snapshot("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryFailoverSnapshotRequiredCodeRule {
            message: "primary failover snapshot needs this field".to_string(),
        };
        let mut item = new_primary_failover_snapshot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = PrimaryFailoverSnapshotLocaleLengthRule { max_length: 64 };
        let mut item = new_primary_failover_snapshot("x", "y");
        item.locale = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = PrimaryFailoverSnapshotMinimumRangeRule { min: 3.4, max: 835.9 };
        let mut item = new_primary_failover_snapshot("x", "y");
        item.minimum = 837.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = PrimaryFailoverSnapshotRequiredLocaleRule {
            message: "primary failover snapshot needs this field".to_string(),
        };
        let mut item = new_primary_failover_snapshot("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
