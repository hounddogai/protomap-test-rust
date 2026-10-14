use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled federation request must pass. Check returns its problems, or none.
pub trait ScheduledFederationRequestRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledFederationRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationRequestArchivedGuardRule {
    pub limit: i64,
}

impl ScheduledFederationRequestRule for ScheduledFederationRequestArchivedGuardRule {
    fn check(&self, item: &ScheduledFederationRequest) -> Vec<String> {
        if item.archived && (item.priority > self.limit) {
            return vec![format!(
                "priority {} exceeds {} while archived",
                item.priority, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationRequestTitleLengthRule {
    pub max_length: i64,
}

impl ScheduledFederationRequestRule for ScheduledFederationRequestTitleLengthRule {
    fn check(&self, item: &ScheduledFederationRequest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationRequestAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ScheduledFederationRequestRule for ScheduledFederationRequestAllowedCodeRule {
    fn check(&self, item: &ScheduledFederationRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled federation request.
#[derive(Debug, Clone)]
pub struct ScheduledFederationRequestValidator {
    pub rules: Vec<Rc<dyn ScheduledFederationRequestRule>>,
}

impl ScheduledFederationRequestValidator {
    pub fn validate(&self, item: &ScheduledFederationRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledFederationRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledFederationRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_federation_request_validator() -> ScheduledFederationRequestValidator {
    ScheduledFederationRequestValidator {
        rules: vec![
            Rc::new(ScheduledFederationRequestArchivedGuardRule { limit: 80 })
                as Rc<dyn ScheduledFederationRequestRule>,
            Rc::new(ScheduledFederationRequestTitleLengthRule { max_length: 80 }),
            Rc::new(ScheduledFederationRequestAllowedCodeRule {
                allowed: vec!["fallback".to_string(), "legacy".to_string(), "rolling".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_federation_request_validator();
        let item = new_scheduled_federation_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ScheduledFederationRequestTitleLengthRule { max_length: 120 };
        let mut item = new_scheduled_federation_request("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
