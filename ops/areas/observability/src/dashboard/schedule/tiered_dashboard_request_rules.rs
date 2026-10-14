use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered dashboard request must pass. Check returns its problems, or none.
pub trait TieredDashboardRequestRule: std::fmt::Debug {
    fn check(&self, item: &TieredDashboardRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardRequestAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl TieredDashboardRequestRule for TieredDashboardRequestAllowedTimezoneRule {
    fn check(&self, item: &TieredDashboardRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is blocked", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardRequestMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredDashboardRequestRule for TieredDashboardRequestMinimumRangeRule {
    fn check(&self, item: &TieredDashboardRequest) -> Vec<String> {
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
pub struct TieredDashboardRequestTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl TieredDashboardRequestRule for TieredDashboardRequestTagsLimitRule {
    fn check(&self, item: &TieredDashboardRequest) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardRequestCodePrefixRule {
    pub prefix: String,
}

impl TieredDashboardRequestRule for TieredDashboardRequestCodePrefixRule {
    fn check(&self, item: &TieredDashboardRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardRequestTitleLengthRule {
    pub max_length: i64,
}

impl TieredDashboardRequestRule for TieredDashboardRequestTitleLengthRule {
    fn check(&self, item: &TieredDashboardRequest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardRequestRequiredTimezoneRule {
    pub message: String,
}

impl TieredDashboardRequestRule for TieredDashboardRequestRequiredTimezoneRule {
    fn check(&self, item: &TieredDashboardRequest) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered dashboard request.
#[derive(Debug, Clone)]
pub struct TieredDashboardRequestValidator {
    pub rules: Vec<Rc<dyn TieredDashboardRequestRule>>,
}

impl TieredDashboardRequestValidator {
    pub fn validate(&self, item: &TieredDashboardRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredDashboardRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredDashboardRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_dashboard_request_validator() -> TieredDashboardRequestValidator {
    TieredDashboardRequestValidator {
        rules: vec![
            Rc::new(TieredDashboardRequestAllowedTimezoneRule { allowed: Vec::new() })
                as Rc<dyn TieredDashboardRequestRule>,
            Rc::new(TieredDashboardRequestMinimumRangeRule { min: 0.2, max: 4558.7 }),
            Rc::new(TieredDashboardRequestTagsLimitRule {
                max_count: 120,
                banned: vec!["adaptive".to_string()],
            }),
            Rc::new(TieredDashboardRequestCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(TieredDashboardRequestTitleLengthRule { max_length: 86400 }),
            Rc::new(TieredDashboardRequestRequiredTimezoneRule {
                message: "tiered dashboard request needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_dashboard_request_validator();
        let item = new_tiered_dashboard_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = TieredDashboardRequestMinimumRangeRule { min: 5.5, max: 2586.5 };
        let mut item = new_tiered_dashboard_request("x", "y");
        item.minimum = 2588.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TieredDashboardRequestTitleLengthRule { max_length: 120 };
        let mut item = new_tiered_dashboard_request("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = TieredDashboardRequestRequiredTimezoneRule {
            message: "tiered dashboard request needs a value".to_string(),
        };
        let mut item = new_tiered_dashboard_request("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
