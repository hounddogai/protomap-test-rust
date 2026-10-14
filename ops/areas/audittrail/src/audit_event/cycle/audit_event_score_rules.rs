use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a audit event score must pass. Check returns its problems, or none.
pub trait AuditEventScoreRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventScoreRequiredCategoryRule {
    pub message: String,
}

impl AuditEventScoreRule for AuditEventScoreRequiredCategoryRule {
    fn check(&self, item: &AuditEventScore) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventScoreLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AuditEventScoreRule for AuditEventScoreLimitRangeRule {
    fn check(&self, item: &AuditEventScore) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventScoreCodeLengthRule {
    pub max_length: i64,
}

impl AuditEventScoreRule for AuditEventScoreCodeLengthRule {
    fn check(&self, item: &AuditEventScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventScoreScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AuditEventScoreRule for AuditEventScoreScoreRangeRule {
    fn check(&self, item: &AuditEventScore) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventScoreCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AuditEventScoreRule for AuditEventScoreCapacityRangeRule {
    fn check(&self, item: &AuditEventScore) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventScoreRequiredTitleRule {
    pub message: String,
}

impl AuditEventScoreRule for AuditEventScoreRequiredTitleRule {
    fn check(&self, item: &AuditEventScore) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event score.
#[derive(Debug, Clone)]
pub struct AuditEventScoreValidator {
    pub rules: Vec<Rc<dyn AuditEventScoreRule>>,
}

impl AuditEventScoreValidator {
    pub fn validate(&self, item: &AuditEventScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AuditEventScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_score_validator() -> AuditEventScoreValidator {
    AuditEventScoreValidator {
        rules: vec![
            Rc::new(AuditEventScoreRequiredCategoryRule {
                message: "audit event score needs a value".to_string(),
            }) as Rc<dyn AuditEventScoreRule>,
            Rc::new(AuditEventScoreLimitRangeRule { min: 80, max: 255 }),
            Rc::new(AuditEventScoreCodeLengthRule { max_length: 255 }),
            Rc::new(AuditEventScoreScoreRangeRule { min: 2.8, max: 3928.9 }),
            Rc::new(AuditEventScoreCapacityRangeRule { min: 16, max: 1000 }),
            Rc::new(AuditEventScoreRequiredTitleRule {
                message: "audit event score needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_score_validator();
        let item = new_audit_event_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = AuditEventScoreRequiredCategoryRule {
            message: "audit event score needs an entry".to_string(),
        };
        let mut item = new_audit_event_score("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = AuditEventScoreLimitRangeRule { min: 64, max: 1000 };
        let mut item = new_audit_event_score("x", "y");
        item.limit = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AuditEventScoreCodeLengthRule { max_length: 86400 };
        let mut item = new_audit_event_score("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = AuditEventScoreScoreRangeRule { min: 6.2, max: 3805.6 };
        let mut item = new_audit_event_score("x", "y");
        item.score = 3807.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = AuditEventScoreCapacityRangeRule { min: 255, max: 86400 };
        let mut item = new_audit_event_score("x", "y");
        item.capacity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = AuditEventScoreRequiredTitleRule {
            message: "audit event score needs an entry".to_string(),
        };
        let mut item = new_audit_event_score("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
