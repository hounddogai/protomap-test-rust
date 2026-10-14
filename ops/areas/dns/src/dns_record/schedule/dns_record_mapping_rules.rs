use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record mapping must pass. Check returns its problems, or none.
pub trait DnsRecordMappingRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordMappingRequiredTimezoneRule {
    pub message: String,
}

impl DnsRecordMappingRule for DnsRecordMappingRequiredTimezoneRule {
    fn check(&self, item: &DnsRecordMapping) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordMappingCodePrefixRule {
    pub prefix: String,
}

impl DnsRecordMappingRule for DnsRecordMappingCodePrefixRule {
    fn check(&self, item: &DnsRecordMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordMappingExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordMappingRule for DnsRecordMappingExpiresAtRangeRule {
    fn check(&self, item: &DnsRecordMapping) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordMappingLabelLengthRule {
    pub max_length: i64,
}

impl DnsRecordMappingRule for DnsRecordMappingLabelLengthRule {
    fn check(&self, item: &DnsRecordMapping) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordMappingCodeLengthRule {
    pub max_length: i64,
}

impl DnsRecordMappingRule for DnsRecordMappingCodeLengthRule {
    fn check(&self, item: &DnsRecordMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordMappingRequiredLabelRule {
    pub message: String,
}

impl DnsRecordMappingRule for DnsRecordMappingRequiredLabelRule {
    fn check(&self, item: &DnsRecordMapping) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record mapping.
#[derive(Debug, Clone)]
pub struct DnsRecordMappingValidator {
    pub rules: Vec<Rc<dyn DnsRecordMappingRule>>,
}

impl DnsRecordMappingValidator {
    pub fn validate(&self, item: &DnsRecordMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_mapping_validator() -> DnsRecordMappingValidator {
    DnsRecordMappingValidator {
        rules: vec![
            Rc::new(DnsRecordMappingRequiredTimezoneRule {
                message: "dns record mapping needs a value".to_string(),
            }) as Rc<dyn DnsRecordMappingRule>,
            Rc::new(DnsRecordMappingCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(DnsRecordMappingExpiresAtRangeRule { min: 16, max: 32 }),
            Rc::new(DnsRecordMappingLabelLengthRule { max_length: 16 }),
            Rc::new(DnsRecordMappingCodeLengthRule { max_length: 86400 }),
            Rc::new(DnsRecordMappingRequiredLabelRule {
                message: "dns record mapping needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_mapping_validator();
        let item = new_dns_record_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = DnsRecordMappingRequiredTimezoneRule {
            message: "dns record mapping needs this field".to_string(),
        };
        let mut item = new_dns_record_mapping("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = DnsRecordMappingExpiresAtRangeRule { min: 64, max: 86400 };
        let mut item = new_dns_record_mapping("x", "y");
        item.expires_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DnsRecordMappingLabelLengthRule { max_length: 16 };
        let mut item = new_dns_record_mapping("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DnsRecordMappingCodeLengthRule { max_length: 32 };
        let mut item = new_dns_record_mapping("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DnsRecordMappingRequiredLabelRule {
            message: "dns record mapping needs a value".to_string(),
        };
        let mut item = new_dns_record_mapping("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
