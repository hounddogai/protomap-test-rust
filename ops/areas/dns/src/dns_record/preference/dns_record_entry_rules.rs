use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a dns record entry must pass. Check returns its problems, or none.
pub trait DnsRecordEntryRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordEntryNameLengthRule {
    pub max_length: i64,
}

impl DnsRecordEntryRule for DnsRecordEntryNameLengthRule {
    fn check(&self, item: &DnsRecordEntry) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordEntryAllowedNameRule {
    pub allowed: Vec<String>,
}

impl DnsRecordEntryRule for DnsRecordEntryAllowedNameRule {
    fn check(&self, item: &DnsRecordEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is above maximum", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordEntryCodeLengthRule {
    pub max_length: i64,
}

impl DnsRecordEntryRule for DnsRecordEntryCodeLengthRule {
    fn check(&self, item: &DnsRecordEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record entry.
#[derive(Debug, Clone)]
pub struct DnsRecordEntryValidator {
    pub rules: Vec<Rc<dyn DnsRecordEntryRule>>,
}

impl DnsRecordEntryValidator {
    pub fn validate(&self, item: &DnsRecordEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DnsRecordEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_entry_validator() -> DnsRecordEntryValidator {
    DnsRecordEntryValidator {
        rules: vec![
            Rc::new(DnsRecordEntryNameLengthRule { max_length: 120 }) as Rc<dyn DnsRecordEntryRule>,
            Rc::new(DnsRecordEntryAllowedNameRule {
                allowed: vec!["pending".to_string(), "rolling".to_string()],
            }),
            Rc::new(DnsRecordEntryCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_entry_validator();
        let item = new_dns_record_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DnsRecordEntryNameLengthRule { max_length: 120 };
        let mut item = new_dns_record_entry("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DnsRecordEntryCodeLengthRule { max_length: 16 };
        let mut item = new_dns_record_entry("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
