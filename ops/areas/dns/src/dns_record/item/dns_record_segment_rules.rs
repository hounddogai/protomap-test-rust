use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record segment must pass. Check returns its problems, or none.
pub trait DnsRecordSegmentRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordSegmentCountryLengthRule {
    pub max_length: i64,
}

impl DnsRecordSegmentRule for DnsRecordSegmentCountryLengthRule {
    fn check(&self, item: &DnsRecordSegment) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordSegmentCodePrefixRule {
    pub prefix: String,
}

impl DnsRecordSegmentRule for DnsRecordSegmentCodePrefixRule {
    fn check(&self, item: &DnsRecordSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordSegmentRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordSegmentRule for DnsRecordSegmentRotationDaysRangeRule {
    fn check(&self, item: &DnsRecordSegment) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordSegmentRequiredCountryRule {
    pub message: String,
}

impl DnsRecordSegmentRule for DnsRecordSegmentRequiredCountryRule {
    fn check(&self, item: &DnsRecordSegment) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordSegmentRequiredTitleRule {
    pub message: String,
}

impl DnsRecordSegmentRule for DnsRecordSegmentRequiredTitleRule {
    fn check(&self, item: &DnsRecordSegment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordSegmentLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordSegmentRule for DnsRecordSegmentLimitRangeRule {
    fn check(&self, item: &DnsRecordSegment) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record segment.
#[derive(Debug, Clone)]
pub struct DnsRecordSegmentValidator {
    pub rules: Vec<Rc<dyn DnsRecordSegmentRule>>,
}

impl DnsRecordSegmentValidator {
    pub fn validate(&self, item: &DnsRecordSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_segment_validator() -> DnsRecordSegmentValidator {
    DnsRecordSegmentValidator {
        rules: vec![
            Rc::new(DnsRecordSegmentCountryLengthRule { max_length: 3600 }) as Rc<dyn DnsRecordSegmentRule>,
            Rc::new(DnsRecordSegmentCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(DnsRecordSegmentRotationDaysRangeRule { min: 16, max: 16 }),
            Rc::new(DnsRecordSegmentRequiredCountryRule {
                message: "dns record segment needs this field".to_string(),
            }),
            Rc::new(DnsRecordSegmentRequiredTitleRule {
                message: "dns record segment needs this field".to_string(),
            }),
            Rc::new(DnsRecordSegmentLimitRangeRule { min: 3600, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_segment_validator();
        let item = new_dns_record_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = DnsRecordSegmentCountryLengthRule { max_length: 3600 };
        let mut item = new_dns_record_segment("x", "y");
        item.country = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = DnsRecordSegmentRotationDaysRangeRule { min: 120, max: 86400 };
        let mut item = new_dns_record_segment("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = DnsRecordSegmentRequiredCountryRule {
            message: "dns record segment needs a value".to_string(),
        };
        let mut item = new_dns_record_segment("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DnsRecordSegmentRequiredTitleRule {
            message: "dns record segment needs a value".to_string(),
        };
        let mut item = new_dns_record_segment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = DnsRecordSegmentLimitRangeRule { min: 255, max: 1000 };
        let mut item = new_dns_record_segment("x", "y");
        item.limit = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
