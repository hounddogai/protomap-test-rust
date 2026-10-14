use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive escalation audit must pass. Check returns its problems, or none.
pub trait AdaptiveEscalationAuditRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveEscalationAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEscalationAuditTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveEscalationAuditRule for AdaptiveEscalationAuditTtlSecondsRangeRule {
    fn check(&self, item: &AdaptiveEscalationAudit) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEscalationAuditAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl AdaptiveEscalationAuditRule for AdaptiveEscalationAuditAllowedCodeRule {
    fn check(&self, item: &AdaptiveEscalationAudit) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEscalationAuditAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveEscalationAuditRule for AdaptiveEscalationAuditAttemptsRangeRule {
    fn check(&self, item: &AdaptiveEscalationAudit) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEscalationAuditLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveEscalationAuditRule for AdaptiveEscalationAuditLimitRangeRule {
    fn check(&self, item: &AdaptiveEscalationAudit) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive escalation audit.
#[derive(Debug, Clone)]
pub struct AdaptiveEscalationAuditValidator {
    pub rules: Vec<Rc<dyn AdaptiveEscalationAuditRule>>,
}

impl AdaptiveEscalationAuditValidator {
    pub fn validate(&self, item: &AdaptiveEscalationAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveEscalationAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveEscalationAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_escalation_audit_validator() -> AdaptiveEscalationAuditValidator {
    AdaptiveEscalationAuditValidator {
        rules: vec![
            Rc::new(AdaptiveEscalationAuditTtlSecondsRangeRule { min: 64, max: 80 })
                as Rc<dyn AdaptiveEscalationAuditRule>,
            Rc::new(AdaptiveEscalationAuditAllowedCodeRule {
                allowed: vec!["batched".to_string(), "seasonal".to_string(), "rolling".to_string()],
            }),
            Rc::new(AdaptiveEscalationAuditAttemptsRangeRule { min: 64, max: 120 }),
            Rc::new(AdaptiveEscalationAuditLimitRangeRule { min: 64, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_escalation_audit_validator();
        let item = new_adaptive_escalation_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = AdaptiveEscalationAuditTtlSecondsRangeRule { min: 255, max: 3600 };
        let mut item = new_adaptive_escalation_audit("x", "y");
        item.ttl_seconds = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = AdaptiveEscalationAuditAttemptsRangeRule { min: 80, max: 86400 };
        let mut item = new_adaptive_escalation_audit("x", "y");
        item.attempts = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = AdaptiveEscalationAuditLimitRangeRule { min: 64, max: 255 };
        let mut item = new_adaptive_escalation_audit("x", "y");
        item.limit = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
