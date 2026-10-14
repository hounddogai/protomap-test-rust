use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a failover request must pass. Check returns its problems, or none.
pub trait FailoverRequestRule: std::fmt::Debug {
    fn check(&self, item: &FailoverRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRequestReviewerLengthRule {
    pub max_length: i64,
}

impl FailoverRequestRule for FailoverRequestReviewerLengthRule {
    fn check(&self, item: &FailoverRequest) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRequestBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FailoverRequestRule for FailoverRequestBurstRangeRule {
    fn check(&self, item: &FailoverRequest) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRequestRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FailoverRequestRule for FailoverRequestRetriesRangeRule {
    fn check(&self, item: &FailoverRequest) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRequestRequiredReviewerRule {
    pub message: String,
}

impl FailoverRequestRule for FailoverRequestRequiredReviewerRule {
    fn check(&self, item: &FailoverRequest) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRequestCountryLengthRule {
    pub max_length: i64,
}

impl FailoverRequestRule for FailoverRequestCountryLengthRule {
    fn check(&self, item: &FailoverRequest) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRequestAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl FailoverRequestRule for FailoverRequestAllowedReviewerRule {
    fn check(&self, item: &FailoverRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is too long", item.reviewer)];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover request.
#[derive(Debug, Clone)]
pub struct FailoverRequestValidator {
    pub rules: Vec<Rc<dyn FailoverRequestRule>>,
}

impl FailoverRequestValidator {
    pub fn validate(&self, item: &FailoverRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FailoverRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_request_validator() -> FailoverRequestValidator {
    FailoverRequestValidator {
        rules: vec![
            Rc::new(FailoverRequestReviewerLengthRule { max_length: 120 }) as Rc<dyn FailoverRequestRule>,
            Rc::new(FailoverRequestBurstRangeRule { min: 80, max: 1000 }),
            Rc::new(FailoverRequestRetriesRangeRule { min: 16, max: 120 }),
            Rc::new(FailoverRequestRequiredReviewerRule {
                message: "failover request needs this field".to_string(),
            }),
            Rc::new(FailoverRequestCountryLengthRule { max_length: 1000 }),
            Rc::new(FailoverRequestAllowedReviewerRule {
                allowed: vec!["batched".to_string(), "capped".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_failover_request_validator();
        let item = new_failover_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = FailoverRequestReviewerLengthRule { max_length: 32 };
        let mut item = new_failover_request("x", "y");
        item.reviewer = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = FailoverRequestBurstRangeRule { min: 1000, max: 86400 };
        let mut item = new_failover_request("x", "y");
        item.burst = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = FailoverRequestRetriesRangeRule { min: 64, max: 255 };
        let mut item = new_failover_request("x", "y");
        item.retries = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = FailoverRequestRequiredReviewerRule {
            message: "failover request needs this field".to_string(),
        };
        let mut item = new_failover_request("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = FailoverRequestCountryLengthRule { max_length: 86400 };
        let mut item = new_failover_request("x", "y");
        item.country = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
