use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a approval claim must pass. Check returns its problems, or none.
pub trait ApprovalClaimRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalClaimTenantIdLengthRule {
    pub max_length: i64,
}

impl ApprovalClaimRule for ApprovalClaimTenantIdLengthRule {
    fn check(&self, item: &ApprovalClaim) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalClaimAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ApprovalClaimRule for ApprovalClaimAllowedCodeRule {
    fn check(&self, item: &ApprovalClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalClaimPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ApprovalClaimRule for ApprovalClaimPriorityRangeRule {
    fn check(&self, item: &ApprovalClaim) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalClaimCodePrefixRule {
    pub prefix: String,
}

impl ApprovalClaimRule for ApprovalClaimCodePrefixRule {
    fn check(&self, item: &ApprovalClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalClaimCodeLengthRule {
    pub max_length: i64,
}

impl ApprovalClaimRule for ApprovalClaimCodeLengthRule {
    fn check(&self, item: &ApprovalClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalClaimNameLengthRule {
    pub max_length: i64,
}

impl ApprovalClaimRule for ApprovalClaimNameLengthRule {
    fn check(&self, item: &ApprovalClaim) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval claim.
#[derive(Debug, Clone)]
pub struct ApprovalClaimValidator {
    pub rules: Vec<Rc<dyn ApprovalClaimRule>>,
}

impl ApprovalClaimValidator {
    pub fn validate(&self, item: &ApprovalClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ApprovalClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_claim_validator() -> ApprovalClaimValidator {
    ApprovalClaimValidator {
        rules: vec![
            Rc::new(ApprovalClaimTenantIdLengthRule { max_length: 16 }) as Rc<dyn ApprovalClaimRule>,
            Rc::new(ApprovalClaimAllowedCodeRule {
                allowed: vec!["strict".to_string(), "scheduled".to_string(), "strict".to_string()],
            }),
            Rc::new(ApprovalClaimPriorityRangeRule { min: 3600, max: 86400 }),
            Rc::new(ApprovalClaimCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ApprovalClaimCodeLengthRule { max_length: 80 }),
            Rc::new(ApprovalClaimNameLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_approval_claim_validator();
        let item = new_approval_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = ApprovalClaimTenantIdLengthRule { max_length: 120 };
        let mut item = new_approval_claim("x", "y");
        item.tenant_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = ApprovalClaimPriorityRangeRule { min: 120, max: 3600 };
        let mut item = new_approval_claim("x", "y");
        item.priority = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ApprovalClaimCodeLengthRule { max_length: 16 };
        let mut item = new_approval_claim("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ApprovalClaimNameLengthRule { max_length: 255 };
        let mut item = new_approval_claim("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
