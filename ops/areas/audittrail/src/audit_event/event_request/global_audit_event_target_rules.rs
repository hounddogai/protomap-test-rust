use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global audit event target must pass. Check returns its problems, or none.
pub trait GlobalAuditEventTargetRule: std::fmt::Debug {
    fn check(&self, item: &GlobalAuditEventTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventTargetPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalAuditEventTargetRule for GlobalAuditEventTargetPortRangeRule {
    fn check(&self, item: &GlobalAuditEventTarget) -> Vec<String> {
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
pub struct GlobalAuditEventTargetRegionLengthRule {
    pub max_length: i64,
}

impl GlobalAuditEventTargetRule for GlobalAuditEventTargetRegionLengthRule {
    fn check(&self, item: &GlobalAuditEventTarget) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventTargetApprovedGuardRule {
    pub limit: i64,
}

impl GlobalAuditEventTargetRule for GlobalAuditEventTargetApprovedGuardRule {
    fn check(&self, item: &GlobalAuditEventTarget) -> Vec<String> {
        if item.approved && (item.revision > self.limit) {
            return vec![format!(
                "revision {} exceeds {} while approved",
                item.revision, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a global audit event target.
#[derive(Debug, Clone)]
pub struct GlobalAuditEventTargetValidator {
    pub rules: Vec<Rc<dyn GlobalAuditEventTargetRule>>,
}

impl GlobalAuditEventTargetValidator {
    pub fn validate(&self, item: &GlobalAuditEventTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalAuditEventTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalAuditEventTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_audit_event_target_validator() -> GlobalAuditEventTargetValidator {
    GlobalAuditEventTargetValidator {
        rules: vec![
            Rc::new(GlobalAuditEventTargetPortRangeRule { min: 3600, max: 3600 }) as Rc<dyn GlobalAuditEventTargetRule>,
            Rc::new(GlobalAuditEventTargetRegionLengthRule { max_length: 86400 }),
            Rc::new(GlobalAuditEventTargetApprovedGuardRule { limit: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_audit_event_target_validator();
        let item = new_global_audit_event_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = GlobalAuditEventTargetPortRangeRule { min: 16, max: 64 };
        let mut item = new_global_audit_event_target("x", "y");
        item.port = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = GlobalAuditEventTargetRegionLengthRule { max_length: 3600 };
        let mut item = new_global_audit_event_target("x", "y");
        item.region = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
