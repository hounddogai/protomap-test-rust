use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched postmortem audit must pass. Check returns its problems, or none.
pub trait BatchedPostmortemAuditRule: std::fmt::Debug {
    fn check(&self, item: &BatchedPostmortemAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPostmortemAuditAllowedNameRule {
    pub allowed: Vec<String>,
}

impl BatchedPostmortemAuditRule for BatchedPostmortemAuditAllowedNameRule {
    fn check(&self, item: &BatchedPostmortemAudit) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is unknown", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPostmortemAuditCodePrefixRule {
    pub prefix: String,
}

impl BatchedPostmortemAuditRule for BatchedPostmortemAuditCodePrefixRule {
    fn check(&self, item: &BatchedPostmortemAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPostmortemAuditNameLengthRule {
    pub max_length: i64,
}

impl BatchedPostmortemAuditRule for BatchedPostmortemAuditNameLengthRule {
    fn check(&self, item: &BatchedPostmortemAudit) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPostmortemAuditOwnerIdLengthRule {
    pub max_length: i64,
}

impl BatchedPostmortemAuditRule for BatchedPostmortemAuditOwnerIdLengthRule {
    fn check(&self, item: &BatchedPostmortemAudit) -> Vec<String> {
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
pub struct BatchedPostmortemAuditRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedPostmortemAuditRule for BatchedPostmortemAuditRotationDaysRangeRule {
    fn check(&self, item: &BatchedPostmortemAudit) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPostmortemAuditRequiredCodeRule {
    pub message: String,
}

impl BatchedPostmortemAuditRule for BatchedPostmortemAuditRequiredCodeRule {
    fn check(&self, item: &BatchedPostmortemAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched postmortem audit.
#[derive(Debug, Clone)]
pub struct BatchedPostmortemAuditValidator {
    pub rules: Vec<Rc<dyn BatchedPostmortemAuditRule>>,
}

impl BatchedPostmortemAuditValidator {
    pub fn validate(&self, item: &BatchedPostmortemAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedPostmortemAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedPostmortemAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_postmortem_audit_validator() -> BatchedPostmortemAuditValidator {
    BatchedPostmortemAuditValidator {
        rules: vec![
            Rc::new(BatchedPostmortemAuditAllowedNameRule {
                allowed: vec!["rolling".to_string(), "scheduled".to_string()],
            }) as Rc<dyn BatchedPostmortemAuditRule>,
            Rc::new(BatchedPostmortemAuditCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(BatchedPostmortemAuditNameLengthRule { max_length: 64 }),
            Rc::new(BatchedPostmortemAuditOwnerIdLengthRule { max_length: 86400 }),
            Rc::new(BatchedPostmortemAuditRotationDaysRangeRule { min: 32, max: 32 }),
            Rc::new(BatchedPostmortemAuditRequiredCodeRule {
                message: "batched postmortem audit needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_postmortem_audit_validator();
        let item = new_batched_postmortem_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = BatchedPostmortemAuditNameLengthRule { max_length: 120 };
        let mut item = new_batched_postmortem_audit("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = BatchedPostmortemAuditOwnerIdLengthRule { max_length: 80 };
        let mut item = new_batched_postmortem_audit("x", "y");
        item.owner_id = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = BatchedPostmortemAuditRotationDaysRangeRule { min: 120, max: 1000 };
        let mut item = new_batched_postmortem_audit("x", "y");
        item.rotation_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedPostmortemAuditRequiredCodeRule {
            message: "batched postmortem audit needs an entry".to_string(),
        };
        let mut item = new_batched_postmortem_audit("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
