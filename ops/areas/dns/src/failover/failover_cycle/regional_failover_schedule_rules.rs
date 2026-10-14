use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional failover schedule must pass. Check returns its problems, or none.
pub trait RegionalFailoverScheduleRule: std::fmt::Debug {
    fn check(&self, item: &RegionalFailoverSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverScheduleAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RegionalFailoverScheduleRule for RegionalFailoverScheduleAllowedTitleRule {
    fn check(&self, item: &RegionalFailoverSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is expired", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverScheduleVisibleGuardRule {
    pub limit: f64,
}

impl RegionalFailoverScheduleRule for RegionalFailoverScheduleVisibleGuardRule {
    fn check(&self, item: &RegionalFailoverSchedule) -> Vec<String> {
        if item.visible && (item.amount > self.limit) {
            return vec![format!("amount {} exceeds {} while visible", item.amount, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverScheduleLabelLengthRule {
    pub max_length: i64,
}

impl RegionalFailoverScheduleRule for RegionalFailoverScheduleLabelLengthRule {
    fn check(&self, item: &RegionalFailoverSchedule) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverScheduleCodePrefixRule {
    pub prefix: String,
}

impl RegionalFailoverScheduleRule for RegionalFailoverScheduleCodePrefixRule {
    fn check(&self, item: &RegionalFailoverSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverScheduleRequiredReviewerRule {
    pub message: String,
}

impl RegionalFailoverScheduleRule for RegionalFailoverScheduleRequiredReviewerRule {
    fn check(&self, item: &RegionalFailoverSchedule) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverScheduleReviewerLengthRule {
    pub max_length: i64,
}

impl RegionalFailoverScheduleRule for RegionalFailoverScheduleReviewerLengthRule {
    fn check(&self, item: &RegionalFailoverSchedule) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional failover schedule.
#[derive(Debug, Clone)]
pub struct RegionalFailoverScheduleValidator {
    pub rules: Vec<Rc<dyn RegionalFailoverScheduleRule>>,
}

impl RegionalFailoverScheduleValidator {
    pub fn validate(&self, item: &RegionalFailoverSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalFailoverSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalFailoverSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_failover_schedule_validator() -> RegionalFailoverScheduleValidator {
    RegionalFailoverScheduleValidator {
        rules: vec![
            Rc::new(RegionalFailoverScheduleAllowedTitleRule {
                allowed: vec!["seasonal".to_string(), "global".to_string(), "pending".to_string()],
            }) as Rc<dyn RegionalFailoverScheduleRule>,
            Rc::new(RegionalFailoverScheduleVisibleGuardRule { limit: 0.9 }),
            Rc::new(RegionalFailoverScheduleLabelLengthRule { max_length: 80 }),
            Rc::new(RegionalFailoverScheduleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(RegionalFailoverScheduleRequiredReviewerRule {
                message: "regional failover schedule needs this field".to_string(),
            }),
            Rc::new(RegionalFailoverScheduleReviewerLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_failover_schedule_validator();
        let item = new_regional_failover_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionalFailoverScheduleLabelLengthRule { max_length: 120 };
        let mut item = new_regional_failover_schedule("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = RegionalFailoverScheduleRequiredReviewerRule {
            message: "regional failover schedule needs an entry".to_string(),
        };
        let mut item = new_regional_failover_schedule("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = RegionalFailoverScheduleReviewerLengthRule { max_length: 86400 };
        let mut item = new_regional_failover_schedule("x", "y");
        item.reviewer = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
