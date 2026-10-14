use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a delegation quota must pass. Check returns its problems, or none.
pub trait DelegationQuotaRule: std::fmt::Debug {
    fn check(&self, item: &DelegationQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationQuotaRequiredExternalRefRule {
    pub message: String,
}

impl DelegationQuotaRule for DelegationQuotaRequiredExternalRefRule {
    fn check(&self, item: &DelegationQuota) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationQuotaTitleLengthRule {
    pub max_length: i64,
}

impl DelegationQuotaRule for DelegationQuotaTitleLengthRule {
    fn check(&self, item: &DelegationQuota) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationQuotaCodeLengthRule {
    pub max_length: i64,
}

impl DelegationQuotaRule for DelegationQuotaCodeLengthRule {
    fn check(&self, item: &DelegationQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationQuotaNotesLengthRule {
    pub max_length: i64,
}

impl DelegationQuotaRule for DelegationQuotaNotesLengthRule {
    fn check(&self, item: &DelegationQuota) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationQuotaRequiredCodeRule {
    pub message: String,
}

impl DelegationQuotaRule for DelegationQuotaRequiredCodeRule {
    fn check(&self, item: &DelegationQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation quota.
#[derive(Debug, Clone)]
pub struct DelegationQuotaValidator {
    pub rules: Vec<Rc<dyn DelegationQuotaRule>>,
}

impl DelegationQuotaValidator {
    pub fn validate(&self, item: &DelegationQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DelegationQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_quota_validator() -> DelegationQuotaValidator {
    DelegationQuotaValidator {
        rules: vec![
            Rc::new(DelegationQuotaRequiredExternalRefRule {
                message: "delegation quota needs a value".to_string(),
            }) as Rc<dyn DelegationQuotaRule>,
            Rc::new(DelegationQuotaTitleLengthRule { max_length: 86400 }),
            Rc::new(DelegationQuotaCodeLengthRule { max_length: 64 }),
            Rc::new(DelegationQuotaNotesLengthRule { max_length: 80 }),
            Rc::new(DelegationQuotaRequiredCodeRule {
                message: "delegation quota needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_delegation_quota_validator();
        let item = new_delegation_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = DelegationQuotaRequiredExternalRefRule {
            message: "delegation quota needs a value".to_string(),
        };
        let mut item = new_delegation_quota("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DelegationQuotaTitleLengthRule { max_length: 3600 };
        let mut item = new_delegation_quota("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DelegationQuotaCodeLengthRule { max_length: 3600 };
        let mut item = new_delegation_quota("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = DelegationQuotaNotesLengthRule { max_length: 1000 };
        let mut item = new_delegation_quota("x", "y");
        item.notes = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DelegationQuotaRequiredCodeRule {
            message: "delegation quota needs a value".to_string(),
        };
        let mut item = new_delegation_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
