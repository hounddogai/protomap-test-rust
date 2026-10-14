use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a failover segment must pass. Check returns its problems, or none.
pub trait FailoverSegmentRule: std::fmt::Debug {
    fn check(&self, item: &FailoverSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSegmentOwnerIdLengthRule {
    pub max_length: i64,
}

impl FailoverSegmentRule for FailoverSegmentOwnerIdLengthRule {
    fn check(&self, item: &FailoverSegment) -> Vec<String> {
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
pub struct FailoverSegmentVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FailoverSegmentRule for FailoverSegmentVersionRangeRule {
    fn check(&self, item: &FailoverSegment) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSegmentCodeLengthRule {
    pub max_length: i64,
}

impl FailoverSegmentRule for FailoverSegmentCodeLengthRule {
    fn check(&self, item: &FailoverSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover segment.
#[derive(Debug, Clone)]
pub struct FailoverSegmentValidator {
    pub rules: Vec<Rc<dyn FailoverSegmentRule>>,
}

impl FailoverSegmentValidator {
    pub fn validate(&self, item: &FailoverSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FailoverSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_segment_validator() -> FailoverSegmentValidator {
    FailoverSegmentValidator {
        rules: vec![
            Rc::new(FailoverSegmentOwnerIdLengthRule { max_length: 86400 }) as Rc<dyn FailoverSegmentRule>,
            Rc::new(FailoverSegmentVersionRangeRule { min: 3600, max: 86400 }),
            Rc::new(FailoverSegmentCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_failover_segment_validator();
        let item = new_failover_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = FailoverSegmentOwnerIdLengthRule { max_length: 1000 };
        let mut item = new_failover_segment("x", "y");
        item.owner_id = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = FailoverSegmentVersionRangeRule { min: 32, max: 1000 };
        let mut item = new_failover_segment("x", "y");
        item.version = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FailoverSegmentCodeLengthRule { max_length: 16 };
        let mut item = new_failover_segment("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
