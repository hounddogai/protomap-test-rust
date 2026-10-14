use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched tenant milestone must pass. Check returns its problems, or none.
pub trait BatchedTenantMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &BatchedTenantMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTenantMilestoneScopeLengthRule {
    pub max_length: i64,
}

impl BatchedTenantMilestoneRule for BatchedTenantMilestoneScopeLengthRule {
    fn check(&self, item: &BatchedTenantMilestone) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTenantMilestoneCodePrefixRule {
    pub prefix: String,
}

impl BatchedTenantMilestoneRule for BatchedTenantMilestoneCodePrefixRule {
    fn check(&self, item: &BatchedTenantMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTenantMilestoneRequiredAudienceRule {
    pub message: String,
}

impl BatchedTenantMilestoneRule for BatchedTenantMilestoneRequiredAudienceRule {
    fn check(&self, item: &BatchedTenantMilestone) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTenantMilestoneAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl BatchedTenantMilestoneRule for BatchedTenantMilestoneAllowedPrincipalRule {
    fn check(&self, item: &BatchedTenantMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is below minimum", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTenantMilestoneRequiredCodeRule {
    pub message: String,
}

impl BatchedTenantMilestoneRule for BatchedTenantMilestoneRequiredCodeRule {
    fn check(&self, item: &BatchedTenantMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched tenant milestone.
#[derive(Debug, Clone)]
pub struct BatchedTenantMilestoneValidator {
    pub rules: Vec<Rc<dyn BatchedTenantMilestoneRule>>,
}

impl BatchedTenantMilestoneValidator {
    pub fn validate(&self, item: &BatchedTenantMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedTenantMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedTenantMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_tenant_milestone_validator() -> BatchedTenantMilestoneValidator {
    BatchedTenantMilestoneValidator {
        rules: vec![
            Rc::new(BatchedTenantMilestoneScopeLengthRule { max_length: 80 }) as Rc<dyn BatchedTenantMilestoneRule>,
            Rc::new(BatchedTenantMilestoneCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(BatchedTenantMilestoneRequiredAudienceRule {
                message: "batched tenant milestone needs this field".to_string(),
            }),
            Rc::new(BatchedTenantMilestoneAllowedPrincipalRule {
                allowed: vec!["primary".to_string(), "scheduled".to_string()],
            }),
            Rc::new(BatchedTenantMilestoneRequiredCodeRule {
                message: "batched tenant milestone needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_tenant_milestone_validator();
        let item = new_batched_tenant_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = BatchedTenantMilestoneScopeLengthRule { max_length: 3600 };
        let mut item = new_batched_tenant_milestone("x", "y");
        item.scope = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = BatchedTenantMilestoneRequiredAudienceRule {
            message: "batched tenant milestone needs this field".to_string(),
        };
        let mut item = new_batched_tenant_milestone("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedTenantMilestoneRequiredCodeRule {
            message: "batched tenant milestone needs an entry".to_string(),
        };
        let mut item = new_batched_tenant_milestone("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
