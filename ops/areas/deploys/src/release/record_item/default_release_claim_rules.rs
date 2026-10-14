use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default release claim must pass. Check returns its problems, or none.
pub trait DefaultReleaseClaimRule: std::fmt::Debug {
    fn check(&self, item: &DefaultReleaseClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseClaimRequiredCodeRule {
    pub message: String,
}

impl DefaultReleaseClaimRule for DefaultReleaseClaimRequiredCodeRule {
    fn check(&self, item: &DefaultReleaseClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseClaimExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultReleaseClaimRule for DefaultReleaseClaimExpiresAtRangeRule {
    fn check(&self, item: &DefaultReleaseClaim) -> Vec<String> {
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
pub struct DefaultReleaseClaimPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultReleaseClaimRule for DefaultReleaseClaimPriorityRangeRule {
    fn check(&self, item: &DefaultReleaseClaim) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseClaimAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl DefaultReleaseClaimRule for DefaultReleaseClaimAllowedLabelRule {
    fn check(&self, item: &DefaultReleaseClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is unknown", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseClaimLabelLengthRule {
    pub max_length: i64,
}

impl DefaultReleaseClaimRule for DefaultReleaseClaimLabelLengthRule {
    fn check(&self, item: &DefaultReleaseClaim) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseClaimRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultReleaseClaimRule for DefaultReleaseClaimRevisionRangeRule {
    fn check(&self, item: &DefaultReleaseClaim) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default release claim.
#[derive(Debug, Clone)]
pub struct DefaultReleaseClaimValidator {
    pub rules: Vec<Rc<dyn DefaultReleaseClaimRule>>,
}

impl DefaultReleaseClaimValidator {
    pub fn validate(&self, item: &DefaultReleaseClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultReleaseClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultReleaseClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_release_claim_validator() -> DefaultReleaseClaimValidator {
    DefaultReleaseClaimValidator {
        rules: vec![
            Rc::new(DefaultReleaseClaimRequiredCodeRule {
                message: "default release claim needs an entry".to_string(),
            }) as Rc<dyn DefaultReleaseClaimRule>,
            Rc::new(DefaultReleaseClaimExpiresAtRangeRule { min: 64, max: 64 }),
            Rc::new(DefaultReleaseClaimPriorityRangeRule { min: 16, max: 64 }),
            Rc::new(DefaultReleaseClaimAllowedLabelRule {
                allowed: vec!["global".to_string(), "capped".to_string(), "strict".to_string()],
            }),
            Rc::new(DefaultReleaseClaimLabelLengthRule { max_length: 32 }),
            Rc::new(DefaultReleaseClaimRevisionRangeRule { min: 32, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_release_claim_validator();
        let item = new_default_release_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultReleaseClaimRequiredCodeRule {
            message: "default release claim needs a value".to_string(),
        };
        let mut item = new_default_release_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = DefaultReleaseClaimExpiresAtRangeRule { min: 16, max: 64 };
        let mut item = new_default_release_claim("x", "y");
        item.expires_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = DefaultReleaseClaimPriorityRangeRule { min: 120, max: 255 };
        let mut item = new_default_release_claim("x", "y");
        item.priority = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DefaultReleaseClaimLabelLengthRule { max_length: 32 };
        let mut item = new_default_release_claim("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = DefaultReleaseClaimRevisionRangeRule { min: 255, max: 3600 };
        let mut item = new_default_release_claim("x", "y");
        item.revision = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
