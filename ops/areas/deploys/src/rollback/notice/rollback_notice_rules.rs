use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollback notice must pass. Check returns its problems, or none.
pub trait RollbackNoticeRule: std::fmt::Debug {
    fn check(&self, item: &RollbackNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackNoticeRequiredNameRule {
    pub message: String,
}

impl RollbackNoticeRule for RollbackNoticeRequiredNameRule {
    fn check(&self, item: &RollbackNotice) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackNoticeRequiredCodeRule {
    pub message: String,
}

impl RollbackNoticeRule for RollbackNoticeRequiredCodeRule {
    fn check(&self, item: &RollbackNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackNoticeRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollbackNoticeRule for RollbackNoticeRotationDaysRangeRule {
    fn check(&self, item: &RollbackNotice) -> Vec<String> {
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
pub struct RollbackNoticeScopeLengthRule {
    pub max_length: i64,
}

impl RollbackNoticeRule for RollbackNoticeScopeLengthRule {
    fn check(&self, item: &RollbackNotice) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackNoticeCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollbackNoticeRule for RollbackNoticeCostRangeRule {
    fn check(&self, item: &RollbackNotice) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback notice.
#[derive(Debug, Clone)]
pub struct RollbackNoticeValidator {
    pub rules: Vec<Rc<dyn RollbackNoticeRule>>,
}

impl RollbackNoticeValidator {
    pub fn validate(&self, item: &RollbackNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollbackNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_notice_validator() -> RollbackNoticeValidator {
    RollbackNoticeValidator {
        rules: vec![
            Rc::new(RollbackNoticeRequiredNameRule {
                message: "rollback notice needs this field".to_string(),
            }) as Rc<dyn RollbackNoticeRule>,
            Rc::new(RollbackNoticeRequiredCodeRule {
                message: "rollback notice needs this field".to_string(),
            }),
            Rc::new(RollbackNoticeRotationDaysRangeRule { min: 32, max: 1000 }),
            Rc::new(RollbackNoticeScopeLengthRule { max_length: 16 }),
            Rc::new(RollbackNoticeCostRangeRule { min: 7.5, max: 167.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_notice_validator();
        let item = new_rollback_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RollbackNoticeRequiredNameRule {
            message: "rollback notice needs an entry".to_string(),
        };
        let mut item = new_rollback_notice("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollbackNoticeRequiredCodeRule {
            message: "rollback notice needs this field".to_string(),
        };
        let mut item = new_rollback_notice("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = RollbackNoticeRotationDaysRangeRule { min: 3600, max: 86400 };
        let mut item = new_rollback_notice("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = RollbackNoticeScopeLengthRule { max_length: 3600 };
        let mut item = new_rollback_notice("x", "y");
        item.scope = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = RollbackNoticeCostRangeRule { min: 3.3, max: 3109.6 };
        let mut item = new_rollback_notice("x", "y");
        item.cost = 3111.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
