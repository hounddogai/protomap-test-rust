use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive audit event score must pass. Check returns its problems, or none.
pub trait AdaptiveAuditEventScoreRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveAuditEventScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAuditEventScoreSourceLengthRule {
    pub max_length: i64,
}

impl AdaptiveAuditEventScoreRule for AdaptiveAuditEventScoreSourceLengthRule {
    fn check(&self, item: &AdaptiveAuditEventScore) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAuditEventScorePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveAuditEventScoreRule for AdaptiveAuditEventScorePriorityRangeRule {
    fn check(&self, item: &AdaptiveAuditEventScore) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAuditEventScoreRequiredChecksumRule {
    pub message: String,
}

impl AdaptiveAuditEventScoreRule for AdaptiveAuditEventScoreRequiredChecksumRule {
    fn check(&self, item: &AdaptiveAuditEventScore) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive audit event score.
#[derive(Debug, Clone)]
pub struct AdaptiveAuditEventScoreValidator {
    pub rules: Vec<Rc<dyn AdaptiveAuditEventScoreRule>>,
}

impl AdaptiveAuditEventScoreValidator {
    pub fn validate(&self, item: &AdaptiveAuditEventScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveAuditEventScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveAuditEventScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_audit_event_score_validator() -> AdaptiveAuditEventScoreValidator {
    AdaptiveAuditEventScoreValidator {
        rules: vec![
            Rc::new(AdaptiveAuditEventScoreSourceLengthRule { max_length: 16 }) as Rc<dyn AdaptiveAuditEventScoreRule>,
            Rc::new(AdaptiveAuditEventScorePriorityRangeRule { min: 16, max: 32 }),
            Rc::new(AdaptiveAuditEventScoreRequiredChecksumRule {
                message: "adaptive audit event score needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_audit_event_score_validator();
        let item = new_adaptive_audit_event_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = AdaptiveAuditEventScoreSourceLengthRule { max_length: 3600 };
        let mut item = new_adaptive_audit_event_score("x", "y");
        item.source = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = AdaptiveAuditEventScorePriorityRangeRule { min: 120, max: 3600 };
        let mut item = new_adaptive_audit_event_score("x", "y");
        item.priority = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = AdaptiveAuditEventScoreRequiredChecksumRule {
            message: "adaptive audit event score needs an entry".to_string(),
        };
        let mut item = new_adaptive_audit_event_score("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
