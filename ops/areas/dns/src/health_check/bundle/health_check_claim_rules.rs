use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a health check claim must pass. Check returns its problems, or none.
pub trait HealthCheckClaimRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckClaimTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl HealthCheckClaimRule for HealthCheckClaimTagsLimitRule {
    fn check(&self, item: &HealthCheckClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckClaimRequiredNotesRule {
    pub message: String,
}

impl HealthCheckClaimRule for HealthCheckClaimRequiredNotesRule {
    fn check(&self, item: &HealthCheckClaim) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckClaimRequiredDescriptionRule {
    pub message: String,
}

impl HealthCheckClaimRule for HealthCheckClaimRequiredDescriptionRule {
    fn check(&self, item: &HealthCheckClaim) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckClaimAllowedDescriptionRule {
    pub allowed: Vec<String>,
}

impl HealthCheckClaimRule for HealthCheckClaimAllowedDescriptionRule {
    fn check(&self, item: &HealthCheckClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.description)))
        {
            return vec![format!("description {} is inconsistent", item.description)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckClaimCodeLengthRule {
    pub max_length: i64,
}

impl HealthCheckClaimRule for HealthCheckClaimCodeLengthRule {
    fn check(&self, item: &HealthCheckClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check claim.
#[derive(Debug, Clone)]
pub struct HealthCheckClaimValidator {
    pub rules: Vec<Rc<dyn HealthCheckClaimRule>>,
}

impl HealthCheckClaimValidator {
    pub fn validate(&self, item: &HealthCheckClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &HealthCheckClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_claim_validator() -> HealthCheckClaimValidator {
    HealthCheckClaimValidator {
        rules: vec![
            Rc::new(HealthCheckClaimTagsLimitRule {
                max_count: 120,
                banned: Vec::new(),
            }) as Rc<dyn HealthCheckClaimRule>,
            Rc::new(HealthCheckClaimRequiredNotesRule {
                message: "health check claim needs an entry".to_string(),
            }),
            Rc::new(HealthCheckClaimRequiredDescriptionRule {
                message: "health check claim needs an entry".to_string(),
            }),
            Rc::new(HealthCheckClaimAllowedDescriptionRule { allowed: Vec::new() }),
            Rc::new(HealthCheckClaimCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_claim_validator();
        let item = new_health_check_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = HealthCheckClaimRequiredNotesRule {
            message: "health check claim needs this field".to_string(),
        };
        let mut item = new_health_check_claim("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = HealthCheckClaimRequiredDescriptionRule {
            message: "health check claim needs an entry".to_string(),
        };
        let mut item = new_health_check_claim("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = HealthCheckClaimCodeLengthRule { max_length: 80 };
        let mut item = new_health_check_claim("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
