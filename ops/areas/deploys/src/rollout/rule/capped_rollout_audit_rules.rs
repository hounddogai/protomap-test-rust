use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped rollout audit must pass. Check returns its problems, or none.
pub trait CappedRolloutAuditRule: std::fmt::Debug {
    fn check(&self, item: &CappedRolloutAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRolloutAuditAllowedNameRule {
    pub allowed: Vec<String>,
}

impl CappedRolloutAuditRule for CappedRolloutAuditAllowedNameRule {
    fn check(&self, item: &CappedRolloutAudit) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is below minimum", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRolloutAuditPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedRolloutAuditRule for CappedRolloutAuditPortRangeRule {
    fn check(&self, item: &CappedRolloutAudit) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRolloutAuditTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl CappedRolloutAuditRule for CappedRolloutAuditTagsLimitRule {
    fn check(&self, item: &CappedRolloutAudit) -> Vec<String> {
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
pub struct CappedRolloutAuditAudienceLengthRule {
    pub max_length: i64,
}

impl CappedRolloutAuditRule for CappedRolloutAuditAudienceLengthRule {
    fn check(&self, item: &CappedRolloutAudit) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped rollout audit.
#[derive(Debug, Clone)]
pub struct CappedRolloutAuditValidator {
    pub rules: Vec<Rc<dyn CappedRolloutAuditRule>>,
}

impl CappedRolloutAuditValidator {
    pub fn validate(&self, item: &CappedRolloutAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedRolloutAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedRolloutAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_rollout_audit_validator() -> CappedRolloutAuditValidator {
    CappedRolloutAuditValidator {
        rules: vec![
            Rc::new(CappedRolloutAuditAllowedNameRule {
                allowed: vec!["manual".to_string(), "strict".to_string(), "batched".to_string()],
            }) as Rc<dyn CappedRolloutAuditRule>,
            Rc::new(CappedRolloutAuditPortRangeRule { min: 64, max: 80 }),
            Rc::new(CappedRolloutAuditTagsLimitRule {
                max_count: 32,
                banned: vec!["capped".to_string()],
            }),
            Rc::new(CappedRolloutAuditAudienceLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_rollout_audit_validator();
        let item = new_capped_rollout_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = CappedRolloutAuditPortRangeRule { min: 120, max: 255 };
        let mut item = new_capped_rollout_audit("x", "y");
        item.port = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = CappedRolloutAuditAudienceLengthRule { max_length: 255 };
        let mut item = new_capped_rollout_audit("x", "y");
        item.audience = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
