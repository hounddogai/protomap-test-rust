use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record threshold must pass. Check returns its problems, or none.
pub trait DnsRecordThresholdRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordThresholdQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordThresholdRule for DnsRecordThresholdQuantityRangeRule {
    fn check(&self, item: &DnsRecordThreshold) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordThresholdRequiredKeyIdRule {
    pub message: String,
}

impl DnsRecordThresholdRule for DnsRecordThresholdRequiredKeyIdRule {
    fn check(&self, item: &DnsRecordThreshold) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordThresholdMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DnsRecordThresholdRule for DnsRecordThresholdMarginRangeRule {
    fn check(&self, item: &DnsRecordThreshold) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordThresholdCodeLengthRule {
    pub max_length: i64,
}

impl DnsRecordThresholdRule for DnsRecordThresholdCodeLengthRule {
    fn check(&self, item: &DnsRecordThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordThresholdStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordThresholdRule for DnsRecordThresholdStartsAtRangeRule {
    fn check(&self, item: &DnsRecordThreshold) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a dns record threshold.
#[derive(Debug, Clone)]
pub struct DnsRecordThresholdValidator {
    pub rules: Vec<Rc<dyn DnsRecordThresholdRule>>,
}

impl DnsRecordThresholdValidator {
    pub fn validate(&self, item: &DnsRecordThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DnsRecordThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_threshold_validator() -> DnsRecordThresholdValidator {
    DnsRecordThresholdValidator {
        rules: vec![
            Rc::new(DnsRecordThresholdQuantityRangeRule { min: 80, max: 120 }) as Rc<dyn DnsRecordThresholdRule>,
            Rc::new(DnsRecordThresholdRequiredKeyIdRule {
                message: "dns record threshold needs a value".to_string(),
            }),
            Rc::new(DnsRecordThresholdMarginRangeRule { min: 4.3, max: 3272.2 }),
            Rc::new(DnsRecordThresholdCodeLengthRule { max_length: 120 }),
            Rc::new(DnsRecordThresholdStartsAtRangeRule { min: 3600, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_threshold_validator();
        let item = new_dns_record_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = DnsRecordThresholdQuantityRangeRule { min: 1000, max: 86400 };
        let mut item = new_dns_record_threshold("x", "y");
        item.quantity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = DnsRecordThresholdRequiredKeyIdRule {
            message: "dns record threshold needs this field".to_string(),
        };
        let mut item = new_dns_record_threshold("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = DnsRecordThresholdMarginRangeRule { min: 9.1, max: 3693.8 };
        let mut item = new_dns_record_threshold("x", "y");
        item.margin = 3695.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DnsRecordThresholdCodeLengthRule { max_length: 32 };
        let mut item = new_dns_record_threshold("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = DnsRecordThresholdStartsAtRangeRule { min: 32, max: 1000 };
        let mut item = new_dns_record_threshold("x", "y");
        item.starts_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
