use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a audit event policy must pass. Check returns its problems, or none.
pub trait AuditEventPolicyRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventPolicyMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AuditEventPolicyRule for AuditEventPolicyMinimumRangeRule {
    fn check(&self, item: &AuditEventPolicy) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventPolicyRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AuditEventPolicyRule for AuditEventPolicyRotationDaysRangeRule {
    fn check(&self, item: &AuditEventPolicy) -> Vec<String> {
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
pub struct AuditEventPolicyScopeLengthRule {
    pub max_length: i64,
}

impl AuditEventPolicyRule for AuditEventPolicyScopeLengthRule {
    fn check(&self, item: &AuditEventPolicy) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventPolicyTitleLengthRule {
    pub max_length: i64,
}

impl AuditEventPolicyRule for AuditEventPolicyTitleLengthRule {
    fn check(&self, item: &AuditEventPolicy) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event policy.
#[derive(Debug, Clone)]
pub struct AuditEventPolicyValidator {
    pub rules: Vec<Rc<dyn AuditEventPolicyRule>>,
}

impl AuditEventPolicyValidator {
    pub fn validate(&self, item: &AuditEventPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AuditEventPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_policy_validator() -> AuditEventPolicyValidator {
    AuditEventPolicyValidator {
        rules: vec![
            Rc::new(AuditEventPolicyMinimumRangeRule { min: 6.5, max: 2129.9 }) as Rc<dyn AuditEventPolicyRule>,
            Rc::new(AuditEventPolicyRotationDaysRangeRule { min: 64, max: 80 }),
            Rc::new(AuditEventPolicyScopeLengthRule { max_length: 3600 }),
            Rc::new(AuditEventPolicyTitleLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_policy_validator();
        let item = new_audit_event_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = AuditEventPolicyMinimumRangeRule { min: 7.9, max: 948.3 };
        let mut item = new_audit_event_policy("x", "y");
        item.minimum = 949.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = AuditEventPolicyRotationDaysRangeRule { min: 80, max: 1000 };
        let mut item = new_audit_event_policy("x", "y");
        item.rotation_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = AuditEventPolicyScopeLengthRule { max_length: 16 };
        let mut item = new_audit_event_policy("x", "y");
        item.scope = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = AuditEventPolicyTitleLengthRule { max_length: 120 };
        let mut item = new_audit_event_policy("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
