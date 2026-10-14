use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional peering quota must pass. Check returns its problems, or none.
pub trait RegionalPeeringQuotaRule: std::fmt::Debug {
    fn check(&self, item: &RegionalPeeringQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPeeringQuotaCodeLengthRule {
    pub max_length: i64,
}

impl RegionalPeeringQuotaRule for RegionalPeeringQuotaCodeLengthRule {
    fn check(&self, item: &RegionalPeeringQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPeeringQuotaSegmentLengthRule {
    pub max_length: i64,
}

impl RegionalPeeringQuotaRule for RegionalPeeringQuotaSegmentLengthRule {
    fn check(&self, item: &RegionalPeeringQuota) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPeeringQuotaLabelLengthRule {
    pub max_length: i64,
}

impl RegionalPeeringQuotaRule for RegionalPeeringQuotaLabelLengthRule {
    fn check(&self, item: &RegionalPeeringQuota) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional peering quota.
#[derive(Debug, Clone)]
pub struct RegionalPeeringQuotaValidator {
    pub rules: Vec<Rc<dyn RegionalPeeringQuotaRule>>,
}

impl RegionalPeeringQuotaValidator {
    pub fn validate(&self, item: &RegionalPeeringQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalPeeringQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalPeeringQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_peering_quota_validator() -> RegionalPeeringQuotaValidator {
    RegionalPeeringQuotaValidator {
        rules: vec![
            Rc::new(RegionalPeeringQuotaCodeLengthRule { max_length: 80 }) as Rc<dyn RegionalPeeringQuotaRule>,
            Rc::new(RegionalPeeringQuotaSegmentLengthRule { max_length: 3600 }),
            Rc::new(RegionalPeeringQuotaLabelLengthRule { max_length: 16 }),
        ],
    }
}
