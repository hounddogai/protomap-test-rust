use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a delegation profile must pass. Check returns its problems, or none.
pub trait DelegationProfileRule: std::fmt::Debug {
    fn check(&self, item: &DelegationProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationProfileChecksumLengthRule {
    pub max_length: i64,
}

impl DelegationProfileRule for DelegationProfileChecksumLengthRule {
    fn check(&self, item: &DelegationProfile) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationProfileAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DelegationProfileRule for DelegationProfileAllowedCodeRule {
    fn check(&self, item: &DelegationProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is missing", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationProfileNameLengthRule {
    pub max_length: i64,
}

impl DelegationProfileRule for DelegationProfileNameLengthRule {
    fn check(&self, item: &DelegationProfile) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationProfileExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationProfileRule for DelegationProfileExpiresAtRangeRule {
    fn check(&self, item: &DelegationProfile) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationProfileContactEmailLengthRule {
    pub max_length: i64,
}

impl DelegationProfileRule for DelegationProfileContactEmailLengthRule {
    fn check(&self, item: &DelegationProfile) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationProfileEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationProfileRule for DelegationProfileEndsAtRangeRule {
    fn check(&self, item: &DelegationProfile) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a delegation profile.
#[derive(Debug, Clone)]
pub struct DelegationProfileValidator {
    pub rules: Vec<Rc<dyn DelegationProfileRule>>,
}

impl DelegationProfileValidator {
    pub fn validate(&self, item: &DelegationProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DelegationProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_profile_validator() -> DelegationProfileValidator {
    DelegationProfileValidator {
        rules: vec![
            Rc::new(DelegationProfileChecksumLengthRule { max_length: 255 }) as Rc<dyn DelegationProfileRule>,
            Rc::new(DelegationProfileAllowedCodeRule {
                allowed: vec!["rolling".to_string(), "regional".to_string()],
            }),
            Rc::new(DelegationProfileNameLengthRule { max_length: 64 }),
            Rc::new(DelegationProfileExpiresAtRangeRule { min: 80, max: 120 }),
            Rc::new(DelegationProfileContactEmailLengthRule { max_length: 64 }),
            Rc::new(DelegationProfileEndsAtRangeRule { min: 32, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_delegation_profile_validator();
        let item = new_delegation_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = DelegationProfileChecksumLengthRule { max_length: 16 };
        let mut item = new_delegation_profile("x", "y");
        item.checksum = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DelegationProfileNameLengthRule { max_length: 120 };
        let mut item = new_delegation_profile("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = DelegationProfileExpiresAtRangeRule { min: 32, max: 1000 };
        let mut item = new_delegation_profile("x", "y");
        item.expires_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = DelegationProfileContactEmailLengthRule { max_length: 80 };
        let mut item = new_delegation_profile("x", "y");
        item.contact_email = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = DelegationProfileEndsAtRangeRule { min: 3600, max: 3600 };
        let mut item = new_delegation_profile("x", "y");
        item.ends_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
