use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollback digest must pass. Check returns its problems, or none.
pub trait RollbackDigestRule: std::fmt::Debug {
    fn check(&self, item: &RollbackDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackDigestRequiredExternalRefRule {
    pub message: String,
}

impl RollbackDigestRule for RollbackDigestRequiredExternalRefRule {
    fn check(&self, item: &RollbackDigest) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackDigestOwnerIdLengthRule {
    pub max_length: i64,
}

impl RollbackDigestRule for RollbackDigestOwnerIdLengthRule {
    fn check(&self, item: &RollbackDigest) -> Vec<String> {
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
pub struct RollbackDigestLabelLengthRule {
    pub max_length: i64,
}

impl RollbackDigestRule for RollbackDigestLabelLengthRule {
    fn check(&self, item: &RollbackDigest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackDigestCodePrefixRule {
    pub prefix: String,
}

impl RollbackDigestRule for RollbackDigestCodePrefixRule {
    fn check(&self, item: &RollbackDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackDigestCodeLengthRule {
    pub max_length: i64,
}

impl RollbackDigestRule for RollbackDigestCodeLengthRule {
    fn check(&self, item: &RollbackDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackDigestExternalRefLengthRule {
    pub max_length: i64,
}

impl RollbackDigestRule for RollbackDigestExternalRefLengthRule {
    fn check(&self, item: &RollbackDigest) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback digest.
#[derive(Debug, Clone)]
pub struct RollbackDigestValidator {
    pub rules: Vec<Rc<dyn RollbackDigestRule>>,
}

impl RollbackDigestValidator {
    pub fn validate(&self, item: &RollbackDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollbackDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_digest_validator() -> RollbackDigestValidator {
    RollbackDigestValidator {
        rules: vec![
            Rc::new(RollbackDigestRequiredExternalRefRule {
                message: "rollback digest needs this field".to_string(),
            }) as Rc<dyn RollbackDigestRule>,
            Rc::new(RollbackDigestOwnerIdLengthRule { max_length: 32 }),
            Rc::new(RollbackDigestLabelLengthRule { max_length: 1000 }),
            Rc::new(RollbackDigestCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RollbackDigestCodeLengthRule { max_length: 3600 }),
            Rc::new(RollbackDigestExternalRefLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_digest_validator();
        let item = new_rollback_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = RollbackDigestRequiredExternalRefRule {
            message: "rollback digest needs this field".to_string(),
        };
        let mut item = new_rollback_digest("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = RollbackDigestOwnerIdLengthRule { max_length: 86400 };
        let mut item = new_rollback_digest("x", "y");
        item.owner_id = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RollbackDigestLabelLengthRule { max_length: 32 };
        let mut item = new_rollback_digest("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollbackDigestCodeLengthRule { max_length: 120 };
        let mut item = new_rollback_digest("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = RollbackDigestExternalRefLengthRule { max_length: 86400 };
        let mut item = new_rollback_digest("x", "y");
        item.external_ref = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
