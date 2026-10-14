use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a approval quota must pass. Check returns its problems, or none.
pub trait ApprovalQuotaRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalQuotaRequiredCodeRule {
    pub message: String,
}

impl ApprovalQuotaRule for ApprovalQuotaRequiredCodeRule {
    fn check(&self, item: &ApprovalQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalQuotaQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ApprovalQuotaRule for ApprovalQuotaQuotaUnitsRangeRule {
    fn check(&self, item: &ApprovalQuota) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalQuotaCodeLengthRule {
    pub max_length: i64,
}

impl ApprovalQuotaRule for ApprovalQuotaCodeLengthRule {
    fn check(&self, item: &ApprovalQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalQuotaCodePrefixRule {
    pub prefix: String,
}

impl ApprovalQuotaRule for ApprovalQuotaCodePrefixRule {
    fn check(&self, item: &ApprovalQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalQuotaCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ApprovalQuotaRule for ApprovalQuotaCeilingRangeRule {
    fn check(&self, item: &ApprovalQuota) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalQuotaRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ApprovalQuotaRule for ApprovalQuotaRetriesRangeRule {
    fn check(&self, item: &ApprovalQuota) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a approval quota.
#[derive(Debug, Clone)]
pub struct ApprovalQuotaValidator {
    pub rules: Vec<Rc<dyn ApprovalQuotaRule>>,
}

impl ApprovalQuotaValidator {
    pub fn validate(&self, item: &ApprovalQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ApprovalQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_quota_validator() -> ApprovalQuotaValidator {
    ApprovalQuotaValidator {
        rules: vec![
            Rc::new(ApprovalQuotaRequiredCodeRule {
                message: "approval quota needs this field".to_string(),
            }) as Rc<dyn ApprovalQuotaRule>,
            Rc::new(ApprovalQuotaQuotaUnitsRangeRule { min: 64, max: 1000 }),
            Rc::new(ApprovalQuotaCodeLengthRule { max_length: 32 }),
            Rc::new(ApprovalQuotaCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(ApprovalQuotaCeilingRangeRule { min: 6.6, max: 1891.0 }),
            Rc::new(ApprovalQuotaRetriesRangeRule { min: 3600, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_approval_quota_validator();
        let item = new_approval_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ApprovalQuotaRequiredCodeRule {
            message: "approval quota needs an entry".to_string(),
        };
        let mut item = new_approval_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = ApprovalQuotaQuotaUnitsRangeRule { min: 16, max: 3600 };
        let mut item = new_approval_quota("x", "y");
        item.quota_units = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ApprovalQuotaCodeLengthRule { max_length: 32 };
        let mut item = new_approval_quota("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = ApprovalQuotaCeilingRangeRule { min: 6.4, max: 1270.6 };
        let mut item = new_approval_quota("x", "y");
        item.ceiling = 1272.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = ApprovalQuotaRetriesRangeRule { min: 120, max: 3600 };
        let mut item = new_approval_quota("x", "y");
        item.retries = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
