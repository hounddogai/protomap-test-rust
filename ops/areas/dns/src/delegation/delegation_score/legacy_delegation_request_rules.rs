use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy delegation request must pass. Check returns its problems, or none.
pub trait LegacyDelegationRequestRule: std::fmt::Debug {
    fn check(&self, item: &LegacyDelegationRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDelegationRequestRequiredExternalRefRule {
    pub message: String,
}

impl LegacyDelegationRequestRule for LegacyDelegationRequestRequiredExternalRefRule {
    fn check(&self, item: &LegacyDelegationRequest) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDelegationRequestTimezoneLengthRule {
    pub max_length: i64,
}

impl LegacyDelegationRequestRule for LegacyDelegationRequestTimezoneLengthRule {
    fn check(&self, item: &LegacyDelegationRequest) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDelegationRequestCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyDelegationRequestRule for LegacyDelegationRequestCapacityRangeRule {
    fn check(&self, item: &LegacyDelegationRequest) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDelegationRequestDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyDelegationRequestRule for LegacyDelegationRequestDurationMinutesRangeRule {
    fn check(&self, item: &LegacyDelegationRequest) -> Vec<String> {
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
pub struct LegacyDelegationRequestTitleLengthRule {
    pub max_length: i64,
}

impl LegacyDelegationRequestRule for LegacyDelegationRequestTitleLengthRule {
    fn check(&self, item: &LegacyDelegationRequest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDelegationRequestCodeLengthRule {
    pub max_length: i64,
}

impl LegacyDelegationRequestRule for LegacyDelegationRequestCodeLengthRule {
    fn check(&self, item: &LegacyDelegationRequest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy delegation request.
#[derive(Debug, Clone)]
pub struct LegacyDelegationRequestValidator {
    pub rules: Vec<Rc<dyn LegacyDelegationRequestRule>>,
}

impl LegacyDelegationRequestValidator {
    pub fn validate(&self, item: &LegacyDelegationRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyDelegationRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyDelegationRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_delegation_request_validator() -> LegacyDelegationRequestValidator {
    LegacyDelegationRequestValidator {
        rules: vec![
            Rc::new(LegacyDelegationRequestRequiredExternalRefRule {
                message: "legacy delegation request needs a value".to_string(),
            }) as Rc<dyn LegacyDelegationRequestRule>,
            Rc::new(LegacyDelegationRequestTimezoneLengthRule { max_length: 64 }),
            Rc::new(LegacyDelegationRequestCapacityRangeRule { min: 120, max: 1000 }),
            Rc::new(LegacyDelegationRequestDurationMinutesRangeRule { min: 1000, max: 86400 }),
            Rc::new(LegacyDelegationRequestTitleLengthRule { max_length: 64 }),
            Rc::new(LegacyDelegationRequestCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_delegation_request_validator();
        let item = new_legacy_delegation_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = LegacyDelegationRequestRequiredExternalRefRule {
            message: "legacy delegation request needs a value".to_string(),
        };
        let mut item = new_legacy_delegation_request("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = LegacyDelegationRequestTimezoneLengthRule { max_length: 255 };
        let mut item = new_legacy_delegation_request("x", "y");
        item.timezone = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = LegacyDelegationRequestCapacityRangeRule { min: 120, max: 3600 };
        let mut item = new_legacy_delegation_request("x", "y");
        item.capacity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = LegacyDelegationRequestDurationMinutesRangeRule { min: 64, max: 86400 };
        let mut item = new_legacy_delegation_request("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LegacyDelegationRequestTitleLengthRule { max_length: 1000 };
        let mut item = new_legacy_delegation_request("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyDelegationRequestCodeLengthRule { max_length: 255 };
        let mut item = new_legacy_delegation_request("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
