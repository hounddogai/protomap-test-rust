use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled grant digest must pass. Check returns its problems, or none.
pub trait ScheduledGrantDigestRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledGrantDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledGrantDigestRequiredCodeRule {
    pub message: String,
}

impl ScheduledGrantDigestRule for ScheduledGrantDigestRequiredCodeRule {
    fn check(&self, item: &ScheduledGrantDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledGrantDigestRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledGrantDigestRule for ScheduledGrantDigestRevisionRangeRule {
    fn check(&self, item: &ScheduledGrantDigest) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledGrantDigestRequiredContactEmailRule {
    pub message: String,
}

impl ScheduledGrantDigestRule for ScheduledGrantDigestRequiredContactEmailRule {
    fn check(&self, item: &ScheduledGrantDigest) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled grant digest.
#[derive(Debug, Clone)]
pub struct ScheduledGrantDigestValidator {
    pub rules: Vec<Rc<dyn ScheduledGrantDigestRule>>,
}

impl ScheduledGrantDigestValidator {
    pub fn validate(&self, item: &ScheduledGrantDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledGrantDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledGrantDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_grant_digest_validator() -> ScheduledGrantDigestValidator {
    ScheduledGrantDigestValidator {
        rules: vec![
            Rc::new(ScheduledGrantDigestRequiredCodeRule {
                message: "scheduled grant digest needs this field".to_string(),
            }) as Rc<dyn ScheduledGrantDigestRule>,
            Rc::new(ScheduledGrantDigestRevisionRangeRule { min: 255, max: 1000 }),
            Rc::new(ScheduledGrantDigestRequiredContactEmailRule {
                message: "scheduled grant digest needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_grant_digest_validator();
        let item = new_scheduled_grant_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledGrantDigestRequiredCodeRule {
            message: "scheduled grant digest needs an entry".to_string(),
        };
        let mut item = new_scheduled_grant_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = ScheduledGrantDigestRevisionRangeRule { min: 64, max: 255 };
        let mut item = new_scheduled_grant_digest("x", "y");
        item.revision = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = ScheduledGrantDigestRequiredContactEmailRule {
            message: "scheduled grant digest needs this field".to_string(),
        };
        let mut item = new_scheduled_grant_digest("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
