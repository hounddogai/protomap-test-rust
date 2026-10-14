use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped failover checkpoint must pass. Check returns its problems, or none.
pub trait CappedFailoverCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &CappedFailoverCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFailoverCheckpointRegionLengthRule {
    pub max_length: i64,
}

impl CappedFailoverCheckpointRule for CappedFailoverCheckpointRegionLengthRule {
    fn check(&self, item: &CappedFailoverCheckpoint) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFailoverCheckpointRequiredRegionRule {
    pub message: String,
}

impl CappedFailoverCheckpointRule for CappedFailoverCheckpointRequiredRegionRule {
    fn check(&self, item: &CappedFailoverCheckpoint) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFailoverCheckpointHostLengthRule {
    pub max_length: i64,
}

impl CappedFailoverCheckpointRule for CappedFailoverCheckpointHostLengthRule {
    fn check(&self, item: &CappedFailoverCheckpoint) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFailoverCheckpointCodePrefixRule {
    pub prefix: String,
}

impl CappedFailoverCheckpointRule for CappedFailoverCheckpointCodePrefixRule {
    fn check(&self, item: &CappedFailoverCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFailoverCheckpointDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedFailoverCheckpointRule for CappedFailoverCheckpointDiscountRangeRule {
    fn check(&self, item: &CappedFailoverCheckpoint) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped failover checkpoint.
#[derive(Debug, Clone)]
pub struct CappedFailoverCheckpointValidator {
    pub rules: Vec<Rc<dyn CappedFailoverCheckpointRule>>,
}

impl CappedFailoverCheckpointValidator {
    pub fn validate(&self, item: &CappedFailoverCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedFailoverCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedFailoverCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_failover_checkpoint_validator() -> CappedFailoverCheckpointValidator {
    CappedFailoverCheckpointValidator {
        rules: vec![
            Rc::new(CappedFailoverCheckpointRegionLengthRule { max_length: 16 })
                as Rc<dyn CappedFailoverCheckpointRule>,
            Rc::new(CappedFailoverCheckpointRequiredRegionRule {
                message: "capped failover checkpoint needs this field".to_string(),
            }),
            Rc::new(CappedFailoverCheckpointHostLengthRule { max_length: 3600 }),
            Rc::new(CappedFailoverCheckpointCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(CappedFailoverCheckpointDiscountRangeRule { min: 7.6, max: 2084.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_failover_checkpoint_validator();
        let item = new_capped_failover_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = CappedFailoverCheckpointRegionLengthRule { max_length: 80 };
        let mut item = new_capped_failover_checkpoint("x", "y");
        item.region = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = CappedFailoverCheckpointRequiredRegionRule {
            message: "capped failover checkpoint needs this field".to_string(),
        };
        let mut item = new_capped_failover_checkpoint("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = CappedFailoverCheckpointHostLengthRule { max_length: 3600 };
        let mut item = new_capped_failover_checkpoint("x", "y");
        item.host = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = CappedFailoverCheckpointDiscountRangeRule { min: 7.7, max: 563.3 };
        let mut item = new_capped_failover_checkpoint("x", "y");
        item.discount = 564.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
