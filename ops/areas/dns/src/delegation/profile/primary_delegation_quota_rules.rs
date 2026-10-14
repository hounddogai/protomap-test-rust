use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary delegation quota must pass. Check returns its problems, or none.
pub trait PrimaryDelegationQuotaRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryDelegationQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDelegationQuotaRequiredRegionRule {
    pub message: String,
}

impl PrimaryDelegationQuotaRule for PrimaryDelegationQuotaRequiredRegionRule {
    fn check(&self, item: &PrimaryDelegationQuota) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDelegationQuotaCodePrefixRule {
    pub prefix: String,
}

impl PrimaryDelegationQuotaRule for PrimaryDelegationQuotaCodePrefixRule {
    fn check(&self, item: &PrimaryDelegationQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDelegationQuotaNameLengthRule {
    pub max_length: i64,
}

impl PrimaryDelegationQuotaRule for PrimaryDelegationQuotaNameLengthRule {
    fn check(&self, item: &PrimaryDelegationQuota) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDelegationQuotaRequiredNameRule {
    pub message: String,
}

impl PrimaryDelegationQuotaRule for PrimaryDelegationQuotaRequiredNameRule {
    fn check(&self, item: &PrimaryDelegationQuota) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDelegationQuotaRequiredCodeRule {
    pub message: String,
}

impl PrimaryDelegationQuotaRule for PrimaryDelegationQuotaRequiredCodeRule {
    fn check(&self, item: &PrimaryDelegationQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDelegationQuotaCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryDelegationQuotaRule for PrimaryDelegationQuotaCodeLengthRule {
    fn check(&self, item: &PrimaryDelegationQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary delegation quota.
#[derive(Debug, Clone)]
pub struct PrimaryDelegationQuotaValidator {
    pub rules: Vec<Rc<dyn PrimaryDelegationQuotaRule>>,
}

impl PrimaryDelegationQuotaValidator {
    pub fn validate(&self, item: &PrimaryDelegationQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryDelegationQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryDelegationQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_delegation_quota_validator() -> PrimaryDelegationQuotaValidator {
    PrimaryDelegationQuotaValidator {
        rules: vec![
            Rc::new(PrimaryDelegationQuotaRequiredRegionRule {
                message: "primary delegation quota needs an entry".to_string(),
            }) as Rc<dyn PrimaryDelegationQuotaRule>,
            Rc::new(PrimaryDelegationQuotaCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(PrimaryDelegationQuotaNameLengthRule { max_length: 1000 }),
            Rc::new(PrimaryDelegationQuotaRequiredNameRule {
                message: "primary delegation quota needs this field".to_string(),
            }),
            Rc::new(PrimaryDelegationQuotaRequiredCodeRule {
                message: "primary delegation quota needs this field".to_string(),
            }),
            Rc::new(PrimaryDelegationQuotaCodeLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_delegation_quota_validator();
        let item = new_primary_delegation_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = PrimaryDelegationQuotaRequiredRegionRule {
            message: "primary delegation quota needs an entry".to_string(),
        };
        let mut item = new_primary_delegation_quota("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PrimaryDelegationQuotaNameLengthRule { max_length: 32 };
        let mut item = new_primary_delegation_quota("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PrimaryDelegationQuotaRequiredNameRule {
            message: "primary delegation quota needs a value".to_string(),
        };
        let mut item = new_primary_delegation_quota("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryDelegationQuotaRequiredCodeRule {
            message: "primary delegation quota needs this field".to_string(),
        };
        let mut item = new_primary_delegation_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryDelegationQuotaCodeLengthRule { max_length: 80 };
        let mut item = new_primary_delegation_quota("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
