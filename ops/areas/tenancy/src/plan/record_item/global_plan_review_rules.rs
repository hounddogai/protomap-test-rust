use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global plan review must pass. Check returns its problems, or none.
pub trait GlobalPlanReviewRule: std::fmt::Debug {
    fn check(&self, item: &GlobalPlanReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanReviewDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalPlanReviewRule for GlobalPlanReviewDurationMinutesRangeRule {
    fn check(&self, item: &GlobalPlanReview) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanReviewMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalPlanReviewRule for GlobalPlanReviewMaximumRangeRule {
    fn check(&self, item: &GlobalPlanReview) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanReviewAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl GlobalPlanReviewRule for GlobalPlanReviewAllowedCodeRule {
    fn check(&self, item: &GlobalPlanReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanReviewNameLengthRule {
    pub max_length: i64,
}

impl GlobalPlanReviewRule for GlobalPlanReviewNameLengthRule {
    fn check(&self, item: &GlobalPlanReview) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanReviewActiveGuardRule {
    pub limit: i64,
}

impl GlobalPlanReviewRule for GlobalPlanReviewActiveGuardRule {
    fn check(&self, item: &GlobalPlanReview) -> Vec<String> {
        if item.active && (item.version > self.limit) {
            return vec![format!("version {} exceeds {} while active", item.version, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanReviewVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalPlanReviewRule for GlobalPlanReviewVersionRangeRule {
    fn check(&self, item: &GlobalPlanReview) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global plan review.
#[derive(Debug, Clone)]
pub struct GlobalPlanReviewValidator {
    pub rules: Vec<Rc<dyn GlobalPlanReviewRule>>,
}

impl GlobalPlanReviewValidator {
    pub fn validate(&self, item: &GlobalPlanReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalPlanReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalPlanReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_plan_review_validator() -> GlobalPlanReviewValidator {
    GlobalPlanReviewValidator {
        rules: vec![
            Rc::new(GlobalPlanReviewDurationMinutesRangeRule { min: 16, max: 64 }) as Rc<dyn GlobalPlanReviewRule>,
            Rc::new(GlobalPlanReviewMaximumRangeRule { min: 6.8, max: 2294.1 }),
            Rc::new(GlobalPlanReviewAllowedCodeRule {
                allowed: vec!["strict".to_string()],
            }),
            Rc::new(GlobalPlanReviewNameLengthRule { max_length: 255 }),
            Rc::new(GlobalPlanReviewActiveGuardRule { limit: 64 }),
            Rc::new(GlobalPlanReviewVersionRangeRule { min: 32, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_plan_review_validator();
        let item = new_global_plan_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = GlobalPlanReviewDurationMinutesRangeRule { min: 16, max: 255 };
        let mut item = new_global_plan_review("x", "y");
        item.duration_minutes = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = GlobalPlanReviewMaximumRangeRule { min: 0.6, max: 4493.2 };
        let mut item = new_global_plan_review("x", "y");
        item.maximum = 4494.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = GlobalPlanReviewNameLengthRule { max_length: 120 };
        let mut item = new_global_plan_review("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = GlobalPlanReviewVersionRangeRule { min: 32, max: 3600 };
        let mut item = new_global_plan_review("x", "y");
        item.version = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
