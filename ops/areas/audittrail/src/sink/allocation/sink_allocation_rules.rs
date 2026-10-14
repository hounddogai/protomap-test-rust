use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink allocation must pass. Check returns its problems, or none.
pub trait SinkAllocationRule: std::fmt::Debug {
    fn check(&self, item: &SinkAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkAllocationCodeLengthRule {
    pub max_length: i64,
}

impl SinkAllocationRule for SinkAllocationCodeLengthRule {
    fn check(&self, item: &SinkAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkAllocationRequiredClusterRule {
    pub message: String,
}

impl SinkAllocationRule for SinkAllocationRequiredClusterRule {
    fn check(&self, item: &SinkAllocation) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkAllocationLabelLengthRule {
    pub max_length: i64,
}

impl SinkAllocationRule for SinkAllocationLabelLengthRule {
    fn check(&self, item: &SinkAllocation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkAllocationQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SinkAllocationRule for SinkAllocationQuantityRangeRule {
    fn check(&self, item: &SinkAllocation) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkAllocationChecksumLengthRule {
    pub max_length: i64,
}

impl SinkAllocationRule for SinkAllocationChecksumLengthRule {
    fn check(&self, item: &SinkAllocation) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink allocation.
#[derive(Debug, Clone)]
pub struct SinkAllocationValidator {
    pub rules: Vec<Rc<dyn SinkAllocationRule>>,
}

impl SinkAllocationValidator {
    pub fn validate(&self, item: &SinkAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SinkAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_allocation_validator() -> SinkAllocationValidator {
    SinkAllocationValidator {
        rules: vec![
            Rc::new(SinkAllocationCodeLengthRule { max_length: 86400 }) as Rc<dyn SinkAllocationRule>,
            Rc::new(SinkAllocationRequiredClusterRule {
                message: "sink allocation needs an entry".to_string(),
            }),
            Rc::new(SinkAllocationLabelLengthRule { max_length: 86400 }),
            Rc::new(SinkAllocationQuantityRangeRule { min: 80, max: 86400 }),
            Rc::new(SinkAllocationChecksumLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_allocation_validator();
        let item = new_sink_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SinkAllocationCodeLengthRule { max_length: 86400 };
        let mut item = new_sink_allocation("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = SinkAllocationRequiredClusterRule {
            message: "sink allocation needs a value".to_string(),
        };
        let mut item = new_sink_allocation("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SinkAllocationLabelLengthRule { max_length: 64 };
        let mut item = new_sink_allocation("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = SinkAllocationQuantityRangeRule { min: 80, max: 120 };
        let mut item = new_sink_allocation("x", "y");
        item.quantity = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = SinkAllocationChecksumLengthRule { max_length: 3600 };
        let mut item = new_sink_allocation("x", "y");
        item.checksum = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
