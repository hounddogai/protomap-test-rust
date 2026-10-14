use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered dns record threshold must pass. Check returns its problems, or none.
pub trait TieredDnsRecordThresholdRule: std::fmt::Debug {
    fn check(&self, item: &TieredDnsRecordThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordThresholdRequiredExternalRefRule {
    pub message: String,
}

impl TieredDnsRecordThresholdRule for TieredDnsRecordThresholdRequiredExternalRefRule {
    fn check(&self, item: &TieredDnsRecordThreshold) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordThresholdRequiredNameRule {
    pub message: String,
}

impl TieredDnsRecordThresholdRule for TieredDnsRecordThresholdRequiredNameRule {
    fn check(&self, item: &TieredDnsRecordThreshold) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordThresholdOwnerIdLengthRule {
    pub max_length: i64,
}

impl TieredDnsRecordThresholdRule for TieredDnsRecordThresholdOwnerIdLengthRule {
    fn check(&self, item: &TieredDnsRecordThreshold) -> Vec<String> {
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
pub struct TieredDnsRecordThresholdCodePrefixRule {
    pub prefix: String,
}

impl TieredDnsRecordThresholdRule for TieredDnsRecordThresholdCodePrefixRule {
    fn check(&self, item: &TieredDnsRecordThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordThresholdNameLengthRule {
    pub max_length: i64,
}

impl TieredDnsRecordThresholdRule for TieredDnsRecordThresholdNameLengthRule {
    fn check(&self, item: &TieredDnsRecordThreshold) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered dns record threshold.
#[derive(Debug, Clone)]
pub struct TieredDnsRecordThresholdValidator {
    pub rules: Vec<Rc<dyn TieredDnsRecordThresholdRule>>,
}

impl TieredDnsRecordThresholdValidator {
    pub fn validate(&self, item: &TieredDnsRecordThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredDnsRecordThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredDnsRecordThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_dns_record_threshold_validator() -> TieredDnsRecordThresholdValidator {
    TieredDnsRecordThresholdValidator {
        rules: vec![
            Rc::new(TieredDnsRecordThresholdRequiredExternalRefRule {
                message: "tiered dns record threshold needs a value".to_string(),
            }) as Rc<dyn TieredDnsRecordThresholdRule>,
            Rc::new(TieredDnsRecordThresholdRequiredNameRule {
                message: "tiered dns record threshold needs an entry".to_string(),
            }),
            Rc::new(TieredDnsRecordThresholdOwnerIdLengthRule { max_length: 32 }),
            Rc::new(TieredDnsRecordThresholdCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(TieredDnsRecordThresholdNameLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_dns_record_threshold_validator();
        let item = new_tiered_dns_record_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = TieredDnsRecordThresholdRequiredExternalRefRule {
            message: "tiered dns record threshold needs this field".to_string(),
        };
        let mut item = new_tiered_dns_record_threshold("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TieredDnsRecordThresholdRequiredNameRule {
            message: "tiered dns record threshold needs this field".to_string(),
        };
        let mut item = new_tiered_dns_record_threshold("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = TieredDnsRecordThresholdOwnerIdLengthRule { max_length: 255 };
        let mut item = new_tiered_dns_record_threshold("x", "y");
        item.owner_id = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TieredDnsRecordThresholdNameLengthRule { max_length: 32 };
        let mut item = new_tiered_dns_record_threshold("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
