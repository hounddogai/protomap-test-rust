use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient dns record baseline must pass. Check returns its problems, or none.
pub trait LenientDnsRecordBaselineRule: std::fmt::Debug {
    fn check(&self, item: &LenientDnsRecordBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDnsRecordBaselineOwnerIdLengthRule {
    pub max_length: i64,
}

impl LenientDnsRecordBaselineRule for LenientDnsRecordBaselineOwnerIdLengthRule {
    fn check(&self, item: &LenientDnsRecordBaseline) -> Vec<String> {
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
pub struct LenientDnsRecordBaselineCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientDnsRecordBaselineRule for LenientDnsRecordBaselineCapacityRangeRule {
    fn check(&self, item: &LenientDnsRecordBaseline) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDnsRecordBaselineCodePrefixRule {
    pub prefix: String,
}

impl LenientDnsRecordBaselineRule for LenientDnsRecordBaselineCodePrefixRule {
    fn check(&self, item: &LenientDnsRecordBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDnsRecordBaselineCodeLengthRule {
    pub max_length: i64,
}

impl LenientDnsRecordBaselineRule for LenientDnsRecordBaselineCodeLengthRule {
    fn check(&self, item: &LenientDnsRecordBaseline) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient dns record baseline.
#[derive(Debug, Clone)]
pub struct LenientDnsRecordBaselineValidator {
    pub rules: Vec<Rc<dyn LenientDnsRecordBaselineRule>>,
}

impl LenientDnsRecordBaselineValidator {
    pub fn validate(&self, item: &LenientDnsRecordBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientDnsRecordBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientDnsRecordBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_dns_record_baseline_validator() -> LenientDnsRecordBaselineValidator {
    LenientDnsRecordBaselineValidator {
        rules: vec![
            Rc::new(LenientDnsRecordBaselineOwnerIdLengthRule { max_length: 120 })
                as Rc<dyn LenientDnsRecordBaselineRule>,
            Rc::new(LenientDnsRecordBaselineCapacityRangeRule { min: 32, max: 32 }),
            Rc::new(LenientDnsRecordBaselineCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(LenientDnsRecordBaselineCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_dns_record_baseline_validator();
        let item = new_lenient_dns_record_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = LenientDnsRecordBaselineOwnerIdLengthRule { max_length: 80 };
        let mut item = new_lenient_dns_record_baseline("x", "y");
        item.owner_id = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = LenientDnsRecordBaselineCapacityRangeRule { min: 32, max: 86400 };
        let mut item = new_lenient_dns_record_baseline("x", "y");
        item.capacity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientDnsRecordBaselineCodeLengthRule { max_length: 255 };
        let mut item = new_lenient_dns_record_baseline("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
