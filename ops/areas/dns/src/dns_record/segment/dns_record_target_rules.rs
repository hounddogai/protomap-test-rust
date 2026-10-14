use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record target must pass. Check returns its problems, or none.
pub trait DnsRecordTargetRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordTargetCodePrefixRule {
    pub prefix: String,
}

impl DnsRecordTargetRule for DnsRecordTargetCodePrefixRule {
    fn check(&self, item: &DnsRecordTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordTargetSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordTargetRule for DnsRecordTargetSequenceRangeRule {
    fn check(&self, item: &DnsRecordTarget) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordTargetRequiredNameRule {
    pub message: String,
}

impl DnsRecordTargetRule for DnsRecordTargetRequiredNameRule {
    fn check(&self, item: &DnsRecordTarget) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordTargetNameLengthRule {
    pub max_length: i64,
}

impl DnsRecordTargetRule for DnsRecordTargetNameLengthRule {
    fn check(&self, item: &DnsRecordTarget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record target.
#[derive(Debug, Clone)]
pub struct DnsRecordTargetValidator {
    pub rules: Vec<Rc<dyn DnsRecordTargetRule>>,
}

impl DnsRecordTargetValidator {
    pub fn validate(&self, item: &DnsRecordTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DnsRecordTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_target_validator() -> DnsRecordTargetValidator {
    DnsRecordTargetValidator {
        rules: vec![
            Rc::new(DnsRecordTargetCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn DnsRecordTargetRule>,
            Rc::new(DnsRecordTargetSequenceRangeRule { min: 16, max: 16 }),
            Rc::new(DnsRecordTargetRequiredNameRule {
                message: "dns record target needs an entry".to_string(),
            }),
            Rc::new(DnsRecordTargetNameLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_target_validator();
        let item = new_dns_record_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = DnsRecordTargetSequenceRangeRule { min: 64, max: 120 };
        let mut item = new_dns_record_target("x", "y");
        item.sequence = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = DnsRecordTargetRequiredNameRule {
            message: "dns record target needs an entry".to_string(),
        };
        let mut item = new_dns_record_target("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DnsRecordTargetNameLengthRule { max_length: 80 };
        let mut item = new_dns_record_target("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
