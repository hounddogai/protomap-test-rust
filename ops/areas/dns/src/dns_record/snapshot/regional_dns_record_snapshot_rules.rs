use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional dns record snapshot must pass. Check returns its problems, or none.
pub trait RegionalDnsRecordSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &RegionalDnsRecordSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDnsRecordSnapshotAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl RegionalDnsRecordSnapshotRule for RegionalDnsRecordSnapshotAllowedTimezoneRule {
    fn check(&self, item: &RegionalDnsRecordSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is duplicated", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDnsRecordSnapshotCodePrefixRule {
    pub prefix: String,
}

impl RegionalDnsRecordSnapshotRule for RegionalDnsRecordSnapshotCodePrefixRule {
    fn check(&self, item: &RegionalDnsRecordSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDnsRecordSnapshotGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalDnsRecordSnapshotRule for RegionalDnsRecordSnapshotGraceDaysRangeRule {
    fn check(&self, item: &RegionalDnsRecordSnapshot) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDnsRecordSnapshotRequiredTenantIdRule {
    pub message: String,
}

impl RegionalDnsRecordSnapshotRule for RegionalDnsRecordSnapshotRequiredTenantIdRule {
    fn check(&self, item: &RegionalDnsRecordSnapshot) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDnsRecordSnapshotVisibleGuardRule {
    pub limit: i64,
}

impl RegionalDnsRecordSnapshotRule for RegionalDnsRecordSnapshotVisibleGuardRule {
    fn check(&self, item: &RegionalDnsRecordSnapshot) -> Vec<String> {
        if item.visible && (item.grace_days > self.limit) {
            return vec![format!(
                "grace days {} exceeds {} while visible",
                item.grace_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDnsRecordSnapshotNameLengthRule {
    pub max_length: i64,
}

impl RegionalDnsRecordSnapshotRule for RegionalDnsRecordSnapshotNameLengthRule {
    fn check(&self, item: &RegionalDnsRecordSnapshot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional dns record snapshot.
#[derive(Debug, Clone)]
pub struct RegionalDnsRecordSnapshotValidator {
    pub rules: Vec<Rc<dyn RegionalDnsRecordSnapshotRule>>,
}

impl RegionalDnsRecordSnapshotValidator {
    pub fn validate(&self, item: &RegionalDnsRecordSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalDnsRecordSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalDnsRecordSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_dns_record_snapshot_validator() -> RegionalDnsRecordSnapshotValidator {
    RegionalDnsRecordSnapshotValidator {
        rules: vec![
            Rc::new(RegionalDnsRecordSnapshotAllowedTimezoneRule {
                allowed: vec!["rolling".to_string(), "adaptive".to_string()],
            }) as Rc<dyn RegionalDnsRecordSnapshotRule>,
            Rc::new(RegionalDnsRecordSnapshotCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RegionalDnsRecordSnapshotGraceDaysRangeRule { min: 80, max: 86400 }),
            Rc::new(RegionalDnsRecordSnapshotRequiredTenantIdRule {
                message: "regional dns record snapshot needs an entry".to_string(),
            }),
            Rc::new(RegionalDnsRecordSnapshotVisibleGuardRule { limit: 16 }),
            Rc::new(RegionalDnsRecordSnapshotNameLengthRule { max_length: 64 }),
        ],
    }
}
