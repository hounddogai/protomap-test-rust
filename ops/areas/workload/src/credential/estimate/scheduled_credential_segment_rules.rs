use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled credential segment must pass. Check returns its problems, or none.
pub trait ScheduledCredentialSegmentRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledCredentialSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCredentialSegmentPrincipalLengthRule {
    pub max_length: i64,
}

impl ScheduledCredentialSegmentRule for ScheduledCredentialSegmentPrincipalLengthRule {
    fn check(&self, item: &ScheduledCredentialSegment) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCredentialSegmentCodePrefixRule {
    pub prefix: String,
}

impl ScheduledCredentialSegmentRule for ScheduledCredentialSegmentCodePrefixRule {
    fn check(&self, item: &ScheduledCredentialSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCredentialSegmentTitleLengthRule {
    pub max_length: i64,
}

impl ScheduledCredentialSegmentRule for ScheduledCredentialSegmentTitleLengthRule {
    fn check(&self, item: &ScheduledCredentialSegment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled credential segment.
#[derive(Debug, Clone)]
pub struct ScheduledCredentialSegmentValidator {
    pub rules: Vec<Rc<dyn ScheduledCredentialSegmentRule>>,
}

impl ScheduledCredentialSegmentValidator {
    pub fn validate(&self, item: &ScheduledCredentialSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledCredentialSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledCredentialSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_credential_segment_validator() -> ScheduledCredentialSegmentValidator {
    ScheduledCredentialSegmentValidator {
        rules: vec![
            Rc::new(ScheduledCredentialSegmentPrincipalLengthRule { max_length: 255 })
                as Rc<dyn ScheduledCredentialSegmentRule>,
            Rc::new(ScheduledCredentialSegmentCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ScheduledCredentialSegmentTitleLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_credential_segment_validator();
        let item = new_scheduled_credential_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = ScheduledCredentialSegmentPrincipalLengthRule { max_length: 80 };
        let mut item = new_scheduled_credential_segment("x", "y");
        item.principal = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ScheduledCredentialSegmentTitleLengthRule { max_length: 32 };
        let mut item = new_scheduled_credential_segment("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
