use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record template must pass. Check returns its problems, or none.
pub trait DnsRecordTemplateRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordTemplateContactEmailLengthRule {
    pub max_length: i64,
}

impl DnsRecordTemplateRule for DnsRecordTemplateContactEmailLengthRule {
    fn check(&self, item: &DnsRecordTemplate) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordTemplateCodeLengthRule {
    pub max_length: i64,
}

impl DnsRecordTemplateRule for DnsRecordTemplateCodeLengthRule {
    fn check(&self, item: &DnsRecordTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordTemplateVisibleGuardRule {
    pub limit: f64,
}

impl DnsRecordTemplateRule for DnsRecordTemplateVisibleGuardRule {
    fn check(&self, item: &DnsRecordTemplate) -> Vec<String> {
        if item.visible && (item.discount > self.limit) {
            return vec![format!(
                "discount {} exceeds {} while visible",
                item.discount, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordTemplateRequiredCodeRule {
    pub message: String,
}

impl DnsRecordTemplateRule for DnsRecordTemplateRequiredCodeRule {
    fn check(&self, item: &DnsRecordTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordTemplateHostLengthRule {
    pub max_length: i64,
}

impl DnsRecordTemplateRule for DnsRecordTemplateHostLengthRule {
    fn check(&self, item: &DnsRecordTemplate) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordTemplateDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DnsRecordTemplateRule for DnsRecordTemplateDiscountRangeRule {
    fn check(&self, item: &DnsRecordTemplate) -> Vec<String> {
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

/// Runs every rule of a dns record template.
#[derive(Debug, Clone)]
pub struct DnsRecordTemplateValidator {
    pub rules: Vec<Rc<dyn DnsRecordTemplateRule>>,
}

impl DnsRecordTemplateValidator {
    pub fn validate(&self, item: &DnsRecordTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_template_validator() -> DnsRecordTemplateValidator {
    DnsRecordTemplateValidator {
        rules: vec![
            Rc::new(DnsRecordTemplateContactEmailLengthRule { max_length: 32 }) as Rc<dyn DnsRecordTemplateRule>,
            Rc::new(DnsRecordTemplateCodeLengthRule { max_length: 1000 }),
            Rc::new(DnsRecordTemplateVisibleGuardRule { limit: 9.2 }),
            Rc::new(DnsRecordTemplateRequiredCodeRule {
                message: "dns record template needs an entry".to_string(),
            }),
            Rc::new(DnsRecordTemplateHostLengthRule { max_length: 1000 }),
            Rc::new(DnsRecordTemplateDiscountRangeRule { min: 5.8, max: 3669.3 }),
        ],
    }
}
