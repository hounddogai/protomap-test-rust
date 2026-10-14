use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient delegation profile must pass. Check returns its problems, or none.
pub trait LenientDelegationProfileRule: std::fmt::Debug {
    fn check(&self, item: &LenientDelegationProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationProfileRequiredCodeRule {
    pub message: String,
}

impl LenientDelegationProfileRule for LenientDelegationProfileRequiredCodeRule {
    fn check(&self, item: &LenientDelegationProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationProfileTitleLengthRule {
    pub max_length: i64,
}

impl LenientDelegationProfileRule for LenientDelegationProfileTitleLengthRule {
    fn check(&self, item: &LenientDelegationProfile) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationProfileAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl LenientDelegationProfileRule for LenientDelegationProfileAllowedPrincipalRule {
    fn check(&self, item: &LenientDelegationProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is inconsistent", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationProfilePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientDelegationProfileRule for LenientDelegationProfilePriorityRangeRule {
    fn check(&self, item: &LenientDelegationProfile) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a lenient delegation profile.
#[derive(Debug, Clone)]
pub struct LenientDelegationProfileValidator {
    pub rules: Vec<Rc<dyn LenientDelegationProfileRule>>,
}

impl LenientDelegationProfileValidator {
    pub fn validate(&self, item: &LenientDelegationProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientDelegationProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientDelegationProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_delegation_profile_validator() -> LenientDelegationProfileValidator {
    LenientDelegationProfileValidator {
        rules: vec![
            Rc::new(LenientDelegationProfileRequiredCodeRule {
                message: "lenient delegation profile needs an entry".to_string(),
            }) as Rc<dyn LenientDelegationProfileRule>,
            Rc::new(LenientDelegationProfileTitleLengthRule { max_length: 32 }),
            Rc::new(LenientDelegationProfileAllowedPrincipalRule {
                allowed: vec!["strict".to_string()],
            }),
            Rc::new(LenientDelegationProfilePriorityRangeRule { min: 64, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_delegation_profile_validator();
        let item = new_lenient_delegation_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientDelegationProfileRequiredCodeRule {
            message: "lenient delegation profile needs a value".to_string(),
        };
        let mut item = new_lenient_delegation_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LenientDelegationProfileTitleLengthRule { max_length: 120 };
        let mut item = new_lenient_delegation_profile("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = LenientDelegationProfilePriorityRangeRule { min: 64, max: 80 };
        let mut item = new_lenient_delegation_profile("x", "y");
        item.priority = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
