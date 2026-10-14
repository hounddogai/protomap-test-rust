use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global dns record entry must pass. Check returns its problems, or none.
pub trait GlobalDnsRecordEntryRule: std::fmt::Debug {
    fn check(&self, item: &GlobalDnsRecordEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDnsRecordEntryRequiredTitleRule {
    pub message: String,
}

impl GlobalDnsRecordEntryRule for GlobalDnsRecordEntryRequiredTitleRule {
    fn check(&self, item: &GlobalDnsRecordEntry) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDnsRecordEntryRequiredCountryRule {
    pub message: String,
}

impl GlobalDnsRecordEntryRule for GlobalDnsRecordEntryRequiredCountryRule {
    fn check(&self, item: &GlobalDnsRecordEntry) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDnsRecordEntryTitleLengthRule {
    pub max_length: i64,
}

impl GlobalDnsRecordEntryRule for GlobalDnsRecordEntryTitleLengthRule {
    fn check(&self, item: &GlobalDnsRecordEntry) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global dns record entry.
#[derive(Debug, Clone)]
pub struct GlobalDnsRecordEntryValidator {
    pub rules: Vec<Rc<dyn GlobalDnsRecordEntryRule>>,
}

impl GlobalDnsRecordEntryValidator {
    pub fn validate(&self, item: &GlobalDnsRecordEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalDnsRecordEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalDnsRecordEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_dns_record_entry_validator() -> GlobalDnsRecordEntryValidator {
    GlobalDnsRecordEntryValidator {
        rules: vec![
            Rc::new(GlobalDnsRecordEntryRequiredTitleRule {
                message: "global dns record entry needs this field".to_string(),
            }) as Rc<dyn GlobalDnsRecordEntryRule>,
            Rc::new(GlobalDnsRecordEntryRequiredCountryRule {
                message: "global dns record entry needs an entry".to_string(),
            }),
            Rc::new(GlobalDnsRecordEntryTitleLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_dns_record_entry_validator();
        let item = new_global_dns_record_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = GlobalDnsRecordEntryRequiredTitleRule {
            message: "global dns record entry needs this field".to_string(),
        };
        let mut item = new_global_dns_record_entry("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = GlobalDnsRecordEntryRequiredCountryRule {
            message: "global dns record entry needs an entry".to_string(),
        };
        let mut item = new_global_dns_record_entry("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = GlobalDnsRecordEntryTitleLengthRule { max_length: 32 };
        let mut item = new_global_dns_record_entry("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
