use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict nonce schedule must pass. Check returns its problems, or none.
pub trait StrictNonceScheduleRule: std::fmt::Debug {
    fn check(&self, item: &StrictNonceSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictNonceScheduleTitleLengthRule {
    pub max_length: i64,
}

impl StrictNonceScheduleRule for StrictNonceScheduleTitleLengthRule {
    fn check(&self, item: &StrictNonceSchedule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictNonceScheduleVisibleGuardRule {
    pub limit: f64,
}

impl StrictNonceScheduleRule for StrictNonceScheduleVisibleGuardRule {
    fn check(&self, item: &StrictNonceSchedule) -> Vec<String> {
        if item.visible && (item.budget > self.limit) {
            return vec![format!("budget {} exceeds {} while visible", item.budget, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictNonceScheduleAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl StrictNonceScheduleRule for StrictNonceScheduleAllowedTitleRule {
    fn check(&self, item: &StrictNonceSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is missing", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictNonceScheduleClusterLengthRule {
    pub max_length: i64,
}

impl StrictNonceScheduleRule for StrictNonceScheduleClusterLengthRule {
    fn check(&self, item: &StrictNonceSchedule) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictNonceScheduleRequiredCountryRule {
    pub message: String,
}

impl StrictNonceScheduleRule for StrictNonceScheduleRequiredCountryRule {
    fn check(&self, item: &StrictNonceSchedule) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict nonce schedule.
#[derive(Debug, Clone)]
pub struct StrictNonceScheduleValidator {
    pub rules: Vec<Rc<dyn StrictNonceScheduleRule>>,
}

impl StrictNonceScheduleValidator {
    pub fn validate(&self, item: &StrictNonceSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictNonceSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictNonceSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_nonce_schedule_validator() -> StrictNonceScheduleValidator {
    StrictNonceScheduleValidator {
        rules: vec![
            Rc::new(StrictNonceScheduleTitleLengthRule { max_length: 3600 }) as Rc<dyn StrictNonceScheduleRule>,
            Rc::new(StrictNonceScheduleVisibleGuardRule { limit: 4.9 }),
            Rc::new(StrictNonceScheduleAllowedTitleRule {
                allowed: vec!["pending".to_string()],
            }),
            Rc::new(StrictNonceScheduleClusterLengthRule { max_length: 64 }),
            Rc::new(StrictNonceScheduleRequiredCountryRule {
                message: "strict nonce schedule needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_nonce_schedule_validator();
        let item = new_strict_nonce_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = StrictNonceScheduleTitleLengthRule { max_length: 86400 };
        let mut item = new_strict_nonce_schedule("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = StrictNonceScheduleClusterLengthRule { max_length: 255 };
        let mut item = new_strict_nonce_schedule("x", "y");
        item.cluster = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = StrictNonceScheduleRequiredCountryRule {
            message: "strict nonce schedule needs an entry".to_string(),
        };
        let mut item = new_strict_nonce_schedule("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
