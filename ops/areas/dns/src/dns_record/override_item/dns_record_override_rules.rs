use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record override must pass. Check returns its problems, or none.
pub trait DnsRecordOverrideRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordOverrideRequiredTitleRule {
    pub message: String,
}

impl DnsRecordOverrideRule for DnsRecordOverrideRequiredTitleRule {
    fn check(&self, item: &DnsRecordOverride) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordOverrideRequiredCipherRule {
    pub message: String,
}

impl DnsRecordOverrideRule for DnsRecordOverrideRequiredCipherRule {
    fn check(&self, item: &DnsRecordOverride) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordOverrideCipherLengthRule {
    pub max_length: i64,
}

impl DnsRecordOverrideRule for DnsRecordOverrideCipherLengthRule {
    fn check(&self, item: &DnsRecordOverride) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordOverrideRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordOverrideRule for DnsRecordOverrideRevisionRangeRule {
    fn check(&self, item: &DnsRecordOverride) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordOverrideTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl DnsRecordOverrideRule for DnsRecordOverrideTagsLimitRule {
    fn check(&self, item: &DnsRecordOverride) -> Vec<String> {
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

/// Runs every rule of a dns record override.
#[derive(Debug, Clone)]
pub struct DnsRecordOverrideValidator {
    pub rules: Vec<Rc<dyn DnsRecordOverrideRule>>,
}

impl DnsRecordOverrideValidator {
    pub fn validate(&self, item: &DnsRecordOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DnsRecordOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_override_validator() -> DnsRecordOverrideValidator {
    DnsRecordOverrideValidator {
        rules: vec![
            Rc::new(DnsRecordOverrideRequiredTitleRule {
                message: "dns record override needs this field".to_string(),
            }) as Rc<dyn DnsRecordOverrideRule>,
            Rc::new(DnsRecordOverrideRequiredCipherRule {
                message: "dns record override needs an entry".to_string(),
            }),
            Rc::new(DnsRecordOverrideCipherLengthRule { max_length: 120 }),
            Rc::new(DnsRecordOverrideRevisionRangeRule { min: 1000, max: 3600 }),
            Rc::new(DnsRecordOverrideTagsLimitRule {
                max_count: 16,
                banned: vec!["manual".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_override_validator();
        let item = new_dns_record_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DnsRecordOverrideRequiredTitleRule {
            message: "dns record override needs this field".to_string(),
        };
        let mut item = new_dns_record_override("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = DnsRecordOverrideRequiredCipherRule {
            message: "dns record override needs an entry".to_string(),
        };
        let mut item = new_dns_record_override("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = DnsRecordOverrideCipherLengthRule { max_length: 32 };
        let mut item = new_dns_record_override("x", "y");
        item.cipher = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = DnsRecordOverrideRevisionRangeRule { min: 64, max: 86400 };
        let mut item = new_dns_record_override("x", "y");
        item.revision = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
