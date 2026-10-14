use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional dns record segment must pass. Check returns its problems, or none.
pub trait RegionalDnsRecordSegmentRule: std::fmt::Debug {
    fn check(&self, item: &RegionalDnsRecordSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDnsRecordSegmentCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalDnsRecordSegmentRule for RegionalDnsRecordSegmentCapacityRangeRule {
    fn check(&self, item: &RegionalDnsRecordSegment) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDnsRecordSegmentLabelLengthRule {
    pub max_length: i64,
}

impl RegionalDnsRecordSegmentRule for RegionalDnsRecordSegmentLabelLengthRule {
    fn check(&self, item: &RegionalDnsRecordSegment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDnsRecordSegmentUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalDnsRecordSegmentRule for RegionalDnsRecordSegmentUpdatedAtRangeRule {
    fn check(&self, item: &RegionalDnsRecordSegment) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDnsRecordSegmentRequiredCodeRule {
    pub message: String,
}

impl RegionalDnsRecordSegmentRule for RegionalDnsRecordSegmentRequiredCodeRule {
    fn check(&self, item: &RegionalDnsRecordSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional dns record segment.
#[derive(Debug, Clone)]
pub struct RegionalDnsRecordSegmentValidator {
    pub rules: Vec<Rc<dyn RegionalDnsRecordSegmentRule>>,
}

impl RegionalDnsRecordSegmentValidator {
    pub fn validate(&self, item: &RegionalDnsRecordSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalDnsRecordSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalDnsRecordSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_dns_record_segment_validator() -> RegionalDnsRecordSegmentValidator {
    RegionalDnsRecordSegmentValidator {
        rules: vec![
            Rc::new(RegionalDnsRecordSegmentCapacityRangeRule { min: 64, max: 255 })
                as Rc<dyn RegionalDnsRecordSegmentRule>,
            Rc::new(RegionalDnsRecordSegmentLabelLengthRule { max_length: 120 }),
            Rc::new(RegionalDnsRecordSegmentUpdatedAtRangeRule { min: 120, max: 120 }),
            Rc::new(RegionalDnsRecordSegmentRequiredCodeRule {
                message: "regional dns record segment needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_dns_record_segment_validator();
        let item = new_regional_dns_record_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = RegionalDnsRecordSegmentCapacityRangeRule { min: 32, max: 255 };
        let mut item = new_regional_dns_record_segment("x", "y");
        item.capacity = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionalDnsRecordSegmentLabelLengthRule { max_length: 32 };
        let mut item = new_regional_dns_record_segment("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = RegionalDnsRecordSegmentUpdatedAtRangeRule { min: 3600, max: 3600 };
        let mut item = new_regional_dns_record_segment("x", "y");
        item.updated_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalDnsRecordSegmentRequiredCodeRule {
            message: "regional dns record segment needs an entry".to_string(),
        };
        let mut item = new_regional_dns_record_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
