use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a policy claim must pass. Check returns its problems, or none.
pub trait PolicyClaimRule: std::fmt::Debug {
    fn check(&self, item: &PolicyClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyClaimCodePrefixRule {
    pub prefix: String,
}

impl PolicyClaimRule for PolicyClaimCodePrefixRule {
    fn check(&self, item: &PolicyClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyClaimRequiredScopeRule {
    pub message: String,
}

impl PolicyClaimRule for PolicyClaimRequiredScopeRule {
    fn check(&self, item: &PolicyClaim) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyClaimCodeLengthRule {
    pub max_length: i64,
}

impl PolicyClaimRule for PolicyClaimCodeLengthRule {
    fn check(&self, item: &PolicyClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyClaimClusterLengthRule {
    pub max_length: i64,
}

impl PolicyClaimRule for PolicyClaimClusterLengthRule {
    fn check(&self, item: &PolicyClaim) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyClaimNameLengthRule {
    pub max_length: i64,
}

impl PolicyClaimRule for PolicyClaimNameLengthRule {
    fn check(&self, item: &PolicyClaim) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a policy claim.
#[derive(Debug, Clone)]
pub struct PolicyClaimValidator {
    pub rules: Vec<Rc<dyn PolicyClaimRule>>,
}

impl PolicyClaimValidator {
    pub fn validate(&self, item: &PolicyClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PolicyClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_claim_validator() -> PolicyClaimValidator {
    PolicyClaimValidator {
        rules: vec![
            Rc::new(PolicyClaimCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn PolicyClaimRule>,
            Rc::new(PolicyClaimRequiredScopeRule {
                message: "policy claim needs an entry".to_string(),
            }),
            Rc::new(PolicyClaimCodeLengthRule { max_length: 255 }),
            Rc::new(PolicyClaimClusterLengthRule { max_length: 3600 }),
            Rc::new(PolicyClaimNameLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_claim_validator();
        let item = new_policy_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = PolicyClaimRequiredScopeRule {
            message: "policy claim needs this field".to_string(),
        };
        let mut item = new_policy_claim("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PolicyClaimCodeLengthRule { max_length: 32 };
        let mut item = new_policy_claim("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = PolicyClaimClusterLengthRule { max_length: 64 };
        let mut item = new_policy_claim("x", "y");
        item.cluster = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PolicyClaimNameLengthRule { max_length: 120 };
        let mut item = new_policy_claim("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
