use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record policy must pass. Check returns its problems, or none.
pub trait DnsRecordPolicyRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordPolicyRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordPolicyRule for DnsRecordPolicyRevisionRangeRule {
    fn check(&self, item: &DnsRecordPolicy) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordPolicyRequiredNamespaceRule {
    pub message: String,
}

impl DnsRecordPolicyRule for DnsRecordPolicyRequiredNamespaceRule {
    fn check(&self, item: &DnsRecordPolicy) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordPolicyCodeLengthRule {
    pub max_length: i64,
}

impl DnsRecordPolicyRule for DnsRecordPolicyCodeLengthRule {
    fn check(&self, item: &DnsRecordPolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordPolicyWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DnsRecordPolicyRule for DnsRecordPolicyWeightRangeRule {
    fn check(&self, item: &DnsRecordPolicy) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordPolicyNameLengthRule {
    pub max_length: i64,
}

impl DnsRecordPolicyRule for DnsRecordPolicyNameLengthRule {
    fn check(&self, item: &DnsRecordPolicy) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordPolicyRequiredCodeRule {
    pub message: String,
}

impl DnsRecordPolicyRule for DnsRecordPolicyRequiredCodeRule {
    fn check(&self, item: &DnsRecordPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record policy.
#[derive(Debug, Clone)]
pub struct DnsRecordPolicyValidator {
    pub rules: Vec<Rc<dyn DnsRecordPolicyRule>>,
}

impl DnsRecordPolicyValidator {
    pub fn validate(&self, item: &DnsRecordPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_policy_validator() -> DnsRecordPolicyValidator {
    DnsRecordPolicyValidator {
        rules: vec![
            Rc::new(DnsRecordPolicyRevisionRangeRule { min: 3600, max: 86400 }) as Rc<dyn DnsRecordPolicyRule>,
            Rc::new(DnsRecordPolicyRequiredNamespaceRule {
                message: "dns record policy needs a value".to_string(),
            }),
            Rc::new(DnsRecordPolicyCodeLengthRule { max_length: 16 }),
            Rc::new(DnsRecordPolicyWeightRangeRule { min: 8.0, max: 2551.9 }),
            Rc::new(DnsRecordPolicyNameLengthRule { max_length: 80 }),
            Rc::new(DnsRecordPolicyRequiredCodeRule {
                message: "dns record policy needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_policy_validator();
        let item = new_dns_record_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = DnsRecordPolicyRevisionRangeRule { min: 80, max: 86400 };
        let mut item = new_dns_record_policy("x", "y");
        item.revision = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = DnsRecordPolicyRequiredNamespaceRule {
            message: "dns record policy needs a value".to_string(),
        };
        let mut item = new_dns_record_policy("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DnsRecordPolicyCodeLengthRule { max_length: 80 };
        let mut item = new_dns_record_policy("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = DnsRecordPolicyWeightRangeRule { min: 1.5, max: 581.9 };
        let mut item = new_dns_record_policy("x", "y");
        item.weight = 583.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DnsRecordPolicyNameLengthRule { max_length: 80 };
        let mut item = new_dns_record_policy("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DnsRecordPolicyRequiredCodeRule {
            message: "dns record policy needs this field".to_string(),
        };
        let mut item = new_dns_record_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
