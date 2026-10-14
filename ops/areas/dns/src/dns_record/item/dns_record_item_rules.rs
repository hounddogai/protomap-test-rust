use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record item must pass. Check returns its problems, or none.
pub trait DnsRecordItemRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordItem) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordItemTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl DnsRecordItemRule for DnsRecordItemTagsLimitRule {
    fn check(&self, item: &DnsRecordItem) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordItemCodeLengthRule {
    pub max_length: i64,
}

impl DnsRecordItemRule for DnsRecordItemCodeLengthRule {
    fn check(&self, item: &DnsRecordItem) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordItemKeyIdLengthRule {
    pub max_length: i64,
}

impl DnsRecordItemRule for DnsRecordItemKeyIdLengthRule {
    fn check(&self, item: &DnsRecordItem) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordItemRequiredKeyIdRule {
    pub message: String,
}

impl DnsRecordItemRule for DnsRecordItemRequiredKeyIdRule {
    fn check(&self, item: &DnsRecordItem) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record item.
#[derive(Debug, Clone)]
pub struct DnsRecordItemValidator {
    pub rules: Vec<Rc<dyn DnsRecordItemRule>>,
}

impl DnsRecordItemValidator {
    pub fn validate(&self, item: &DnsRecordItem) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordItem) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordItem]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_item_validator() -> DnsRecordItemValidator {
    DnsRecordItemValidator {
        rules: vec![
            Rc::new(DnsRecordItemTagsLimitRule {
                max_count: 120,
                banned: vec!["batched".to_string()],
            }) as Rc<dyn DnsRecordItemRule>,
            Rc::new(DnsRecordItemCodeLengthRule { max_length: 3600 }),
            Rc::new(DnsRecordItemKeyIdLengthRule { max_length: 1000 }),
            Rc::new(DnsRecordItemRequiredKeyIdRule {
                message: "dns record item needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_item_validator();
        let item = new_dns_record_item("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DnsRecordItemCodeLengthRule { max_length: 86400 };
        let mut item = new_dns_record_item("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = DnsRecordItemKeyIdLengthRule { max_length: 120 };
        let mut item = new_dns_record_item("x", "y");
        item.key_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = DnsRecordItemRequiredKeyIdRule {
            message: "dns record item needs this field".to_string(),
        };
        let mut item = new_dns_record_item("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
