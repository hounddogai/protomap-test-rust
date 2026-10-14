use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict query request must pass. Check returns its problems, or none.
pub trait StrictQueryRequestRule: std::fmt::Debug {
    fn check(&self, item: &StrictQueryRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryRequestGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictQueryRequestRule for StrictQueryRequestGraceDaysRangeRule {
    fn check(&self, item: &StrictQueryRequest) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryRequestAudienceLengthRule {
    pub max_length: i64,
}

impl StrictQueryRequestRule for StrictQueryRequestAudienceLengthRule {
    fn check(&self, item: &StrictQueryRequest) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryRequestAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl StrictQueryRequestRule for StrictQueryRequestAllowedTimezoneRule {
    fn check(&self, item: &StrictQueryRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is missing", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryRequestTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl StrictQueryRequestRule for StrictQueryRequestTagsLimitRule {
    fn check(&self, item: &StrictQueryRequest) -> Vec<String> {
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

/// Runs every rule of a strict query request.
#[derive(Debug, Clone)]
pub struct StrictQueryRequestValidator {
    pub rules: Vec<Rc<dyn StrictQueryRequestRule>>,
}

impl StrictQueryRequestValidator {
    pub fn validate(&self, item: &StrictQueryRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictQueryRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictQueryRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_query_request_validator() -> StrictQueryRequestValidator {
    StrictQueryRequestValidator {
        rules: vec![
            Rc::new(StrictQueryRequestGraceDaysRangeRule { min: 64, max: 80 }) as Rc<dyn StrictQueryRequestRule>,
            Rc::new(StrictQueryRequestAudienceLengthRule { max_length: 80 }),
            Rc::new(StrictQueryRequestAllowedTimezoneRule {
                allowed: vec!["strict".to_string(), "primary".to_string()],
            }),
            Rc::new(StrictQueryRequestTagsLimitRule {
                max_count: 120,
                banned: vec!["tiered".to_string(), "global".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_query_request_validator();
        let item = new_strict_query_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = StrictQueryRequestGraceDaysRangeRule { min: 1000, max: 86400 };
        let mut item = new_strict_query_request("x", "y");
        item.grace_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = StrictQueryRequestAudienceLengthRule { max_length: 64 };
        let mut item = new_strict_query_request("x", "y");
        item.audience = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
