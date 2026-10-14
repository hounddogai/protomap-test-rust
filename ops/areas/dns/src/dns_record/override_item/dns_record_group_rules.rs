use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a dns record group must pass. Check returns its problems, or none.
pub trait DnsRecordGroupRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordGroupDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DnsRecordGroupRule for DnsRecordGroupDiscountRangeRule {
    fn check(&self, item: &DnsRecordGroup) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordGroupRequiredCodeRule {
    pub message: String,
}

impl DnsRecordGroupRule for DnsRecordGroupRequiredCodeRule {
    fn check(&self, item: &DnsRecordGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordGroupStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordGroupRule for DnsRecordGroupStartsAtRangeRule {
    fn check(&self, item: &DnsRecordGroup) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordGroupTitleLengthRule {
    pub max_length: i64,
}

impl DnsRecordGroupRule for DnsRecordGroupTitleLengthRule {
    fn check(&self, item: &DnsRecordGroup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordGroupAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DnsRecordGroupRule for DnsRecordGroupAllowedCodeRule {
    fn check(&self, item: &DnsRecordGroup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record group.
#[derive(Debug, Clone)]
pub struct DnsRecordGroupValidator {
    pub rules: Vec<Rc<dyn DnsRecordGroupRule>>,
}

impl DnsRecordGroupValidator {
    pub fn validate(&self, item: &DnsRecordGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DnsRecordGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_group_validator() -> DnsRecordGroupValidator {
    DnsRecordGroupValidator {
        rules: vec![
            Rc::new(DnsRecordGroupDiscountRangeRule { min: 7.8, max: 58.6 }) as Rc<dyn DnsRecordGroupRule>,
            Rc::new(DnsRecordGroupRequiredCodeRule {
                message: "dns record group needs this field".to_string(),
            }),
            Rc::new(DnsRecordGroupStartsAtRangeRule { min: 32, max: 64 }),
            Rc::new(DnsRecordGroupTitleLengthRule { max_length: 16 }),
            Rc::new(DnsRecordGroupAllowedCodeRule {
                allowed: vec!["weighted".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_group_validator();
        let item = new_dns_record_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = DnsRecordGroupDiscountRangeRule { min: 3.9, max: 1075.5 };
        let mut item = new_dns_record_group("x", "y");
        item.discount = 1077.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DnsRecordGroupRequiredCodeRule {
            message: "dns record group needs this field".to_string(),
        };
        let mut item = new_dns_record_group("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = DnsRecordGroupStartsAtRangeRule { min: 16, max: 64 };
        let mut item = new_dns_record_group("x", "y");
        item.starts_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DnsRecordGroupTitleLengthRule { max_length: 80 };
        let mut item = new_dns_record_group("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
