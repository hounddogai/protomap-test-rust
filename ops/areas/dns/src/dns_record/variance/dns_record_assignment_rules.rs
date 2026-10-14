use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a dns record assignment must pass. Check returns its problems, or none.
pub trait DnsRecordAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordAssignmentHostLengthRule {
    pub max_length: i64,
}

impl DnsRecordAssignmentRule for DnsRecordAssignmentHostLengthRule {
    fn check(&self, item: &DnsRecordAssignment) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordAssignmentOwnerIdLengthRule {
    pub max_length: i64,
}

impl DnsRecordAssignmentRule for DnsRecordAssignmentOwnerIdLengthRule {
    fn check(&self, item: &DnsRecordAssignment) -> Vec<String> {
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
pub struct DnsRecordAssignmentAllowedHostRule {
    pub allowed: Vec<String>,
}

impl DnsRecordAssignmentRule for DnsRecordAssignmentAllowedHostRule {
    fn check(&self, item: &DnsRecordAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.host))) {
            return vec![format!("host {} is duplicated", item.host)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordAssignmentLabelLengthRule {
    pub max_length: i64,
}

impl DnsRecordAssignmentRule for DnsRecordAssignmentLabelLengthRule {
    fn check(&self, item: &DnsRecordAssignment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record assignment.
#[derive(Debug, Clone)]
pub struct DnsRecordAssignmentValidator {
    pub rules: Vec<Rc<dyn DnsRecordAssignmentRule>>,
}

impl DnsRecordAssignmentValidator {
    pub fn validate(&self, item: &DnsRecordAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DnsRecordAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_assignment_validator() -> DnsRecordAssignmentValidator {
    DnsRecordAssignmentValidator {
        rules: vec![
            Rc::new(DnsRecordAssignmentHostLengthRule { max_length: 1000 }) as Rc<dyn DnsRecordAssignmentRule>,
            Rc::new(DnsRecordAssignmentOwnerIdLengthRule { max_length: 255 }),
            Rc::new(DnsRecordAssignmentAllowedHostRule {
                allowed: vec!["lenient".to_string(), "adaptive".to_string(), "manual".to_string()],
            }),
            Rc::new(DnsRecordAssignmentLabelLengthRule { max_length: 1000 }),
        ],
    }
}
