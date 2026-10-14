use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal page quota must pass. Check returns its problems, or none.
pub trait SeasonalPageQuotaRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalPageQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPageQuotaClusterLengthRule {
    pub max_length: i64,
}

impl SeasonalPageQuotaRule for SeasonalPageQuotaClusterLengthRule {
    fn check(&self, item: &SeasonalPageQuota) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPageQuotaCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalPageQuotaRule for SeasonalPageQuotaCodeLengthRule {
    fn check(&self, item: &SeasonalPageQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPageQuotaPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalPageQuotaRule for SeasonalPageQuotaPriorityRangeRule {
    fn check(&self, item: &SeasonalPageQuota) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a seasonal page quota.
#[derive(Debug, Clone)]
pub struct SeasonalPageQuotaValidator {
    pub rules: Vec<Rc<dyn SeasonalPageQuotaRule>>,
}

impl SeasonalPageQuotaValidator {
    pub fn validate(&self, item: &SeasonalPageQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalPageQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalPageQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_page_quota_validator() -> SeasonalPageQuotaValidator {
    SeasonalPageQuotaValidator {
        rules: vec![
            Rc::new(SeasonalPageQuotaClusterLengthRule { max_length: 80 }) as Rc<dyn SeasonalPageQuotaRule>,
            Rc::new(SeasonalPageQuotaCodeLengthRule { max_length: 1000 }),
            Rc::new(SeasonalPageQuotaPriorityRangeRule { min: 64, max: 80 }),
        ],
    }
}
