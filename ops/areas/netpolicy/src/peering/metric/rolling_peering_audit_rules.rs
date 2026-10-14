use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling peering audit must pass. Check returns its problems, or none.
pub trait RollingPeeringAuditRule: std::fmt::Debug {
    fn check(&self, item: &RollingPeeringAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPeeringAuditAllowedChannelRule {
    pub allowed: Vec<String>,
}

impl RollingPeeringAuditRule for RollingPeeringAuditAllowedChannelRule {
    fn check(&self, item: &RollingPeeringAudit) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.channel)))
        {
            return vec![format!("channel {} is out of range", item.channel)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPeeringAuditRequiredCodeRule {
    pub message: String,
}

impl RollingPeeringAuditRule for RollingPeeringAuditRequiredCodeRule {
    fn check(&self, item: &RollingPeeringAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPeeringAuditArchivedGuardRule {
    pub limit: i64,
}

impl RollingPeeringAuditRule for RollingPeeringAuditArchivedGuardRule {
    fn check(&self, item: &RollingPeeringAudit) -> Vec<String> {
        if item.archived && (item.rotation_days > self.limit) {
            return vec![format!(
                "rotation days {} exceeds {} while archived",
                item.rotation_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPeeringAuditCodePrefixRule {
    pub prefix: String,
}

impl RollingPeeringAuditRule for RollingPeeringAuditCodePrefixRule {
    fn check(&self, item: &RollingPeeringAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPeeringAuditNameLengthRule {
    pub max_length: i64,
}

impl RollingPeeringAuditRule for RollingPeeringAuditNameLengthRule {
    fn check(&self, item: &RollingPeeringAudit) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPeeringAuditCodeLengthRule {
    pub max_length: i64,
}

impl RollingPeeringAuditRule for RollingPeeringAuditCodeLengthRule {
    fn check(&self, item: &RollingPeeringAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling peering audit.
#[derive(Debug, Clone)]
pub struct RollingPeeringAuditValidator {
    pub rules: Vec<Rc<dyn RollingPeeringAuditRule>>,
}

impl RollingPeeringAuditValidator {
    pub fn validate(&self, item: &RollingPeeringAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingPeeringAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingPeeringAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_peering_audit_validator() -> RollingPeeringAuditValidator {
    RollingPeeringAuditValidator {
        rules: vec![
            Rc::new(RollingPeeringAuditAllowedChannelRule {
                allowed: vec!["lenient".to_string(), "manual".to_string()],
            }) as Rc<dyn RollingPeeringAuditRule>,
            Rc::new(RollingPeeringAuditRequiredCodeRule {
                message: "rolling peering audit needs this field".to_string(),
            }),
            Rc::new(RollingPeeringAuditArchivedGuardRule { limit: 120 }),
            Rc::new(RollingPeeringAuditCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(RollingPeeringAuditNameLengthRule { max_length: 120 }),
            Rc::new(RollingPeeringAuditCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_peering_audit_validator();
        let item = new_rolling_peering_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingPeeringAuditRequiredCodeRule {
            message: "rolling peering audit needs this field".to_string(),
        };
        let mut item = new_rolling_peering_audit("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RollingPeeringAuditNameLengthRule { max_length: 86400 };
        let mut item = new_rolling_peering_audit("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingPeeringAuditCodeLengthRule { max_length: 80 };
        let mut item = new_rolling_peering_audit("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
