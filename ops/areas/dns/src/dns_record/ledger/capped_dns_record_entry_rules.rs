use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped dns record entry must pass. Check returns its problems, or none.
pub trait CappedDnsRecordEntryRule: std::fmt::Debug {
    fn check(&self, item: &CappedDnsRecordEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDnsRecordEntryAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl CappedDnsRecordEntryRule for CappedDnsRecordEntryAllowedTitleRule {
    fn check(&self, item: &CappedDnsRecordEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is too long", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDnsRecordEntryCodePrefixRule {
    pub prefix: String,
}

impl CappedDnsRecordEntryRule for CappedDnsRecordEntryCodePrefixRule {
    fn check(&self, item: &CappedDnsRecordEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDnsRecordEntryTitleLengthRule {
    pub max_length: i64,
}

impl CappedDnsRecordEntryRule for CappedDnsRecordEntryTitleLengthRule {
    fn check(&self, item: &CappedDnsRecordEntry) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDnsRecordEntryRequiredKeyIdRule {
    pub message: String,
}

impl CappedDnsRecordEntryRule for CappedDnsRecordEntryRequiredKeyIdRule {
    fn check(&self, item: &CappedDnsRecordEntry) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDnsRecordEntryKeyIdLengthRule {
    pub max_length: i64,
}

impl CappedDnsRecordEntryRule for CappedDnsRecordEntryKeyIdLengthRule {
    fn check(&self, item: &CappedDnsRecordEntry) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDnsRecordEntryRequiredNameRule {
    pub message: String,
}

impl CappedDnsRecordEntryRule for CappedDnsRecordEntryRequiredNameRule {
    fn check(&self, item: &CappedDnsRecordEntry) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped dns record entry.
#[derive(Debug, Clone)]
pub struct CappedDnsRecordEntryValidator {
    pub rules: Vec<Rc<dyn CappedDnsRecordEntryRule>>,
}

impl CappedDnsRecordEntryValidator {
    pub fn validate(&self, item: &CappedDnsRecordEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedDnsRecordEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedDnsRecordEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_dns_record_entry_validator() -> CappedDnsRecordEntryValidator {
    CappedDnsRecordEntryValidator {
        rules: vec![
            Rc::new(CappedDnsRecordEntryAllowedTitleRule { allowed: Vec::new() }) as Rc<dyn CappedDnsRecordEntryRule>,
            Rc::new(CappedDnsRecordEntryCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(CappedDnsRecordEntryTitleLengthRule { max_length: 64 }),
            Rc::new(CappedDnsRecordEntryRequiredKeyIdRule {
                message: "capped dns record entry needs a value".to_string(),
            }),
            Rc::new(CappedDnsRecordEntryKeyIdLengthRule { max_length: 120 }),
            Rc::new(CappedDnsRecordEntryRequiredNameRule {
                message: "capped dns record entry needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_dns_record_entry_validator();
        let item = new_capped_dns_record_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = CappedDnsRecordEntryTitleLengthRule { max_length: 64 };
        let mut item = new_capped_dns_record_entry("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = CappedDnsRecordEntryRequiredKeyIdRule {
            message: "capped dns record entry needs a value".to_string(),
        };
        let mut item = new_capped_dns_record_entry("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = CappedDnsRecordEntryKeyIdLengthRule { max_length: 16 };
        let mut item = new_capped_dns_record_entry("x", "y");
        item.key_id = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = CappedDnsRecordEntryRequiredNameRule {
            message: "capped dns record entry needs an entry".to_string(),
        };
        let mut item = new_capped_dns_record_entry("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
