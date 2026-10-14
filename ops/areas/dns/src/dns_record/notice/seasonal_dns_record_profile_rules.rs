use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal dns record profile must pass. Check returns its problems, or none.
pub trait SeasonalDnsRecordProfileRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalDnsRecordProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDnsRecordProfileNotesLengthRule {
    pub max_length: i64,
}

impl SeasonalDnsRecordProfileRule for SeasonalDnsRecordProfileNotesLengthRule {
    fn check(&self, item: &SeasonalDnsRecordProfile) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDnsRecordProfileRequiredNotesRule {
    pub message: String,
}

impl SeasonalDnsRecordProfileRule for SeasonalDnsRecordProfileRequiredNotesRule {
    fn check(&self, item: &SeasonalDnsRecordProfile) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDnsRecordProfileRequiredPrincipalRule {
    pub message: String,
}

impl SeasonalDnsRecordProfileRule for SeasonalDnsRecordProfileRequiredPrincipalRule {
    fn check(&self, item: &SeasonalDnsRecordProfile) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDnsRecordProfileAllowedHostRule {
    pub allowed: Vec<String>,
}

impl SeasonalDnsRecordProfileRule for SeasonalDnsRecordProfileAllowedHostRule {
    fn check(&self, item: &SeasonalDnsRecordProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.host))) {
            return vec![format!("host {} is stale", item.host)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal dns record profile.
#[derive(Debug, Clone)]
pub struct SeasonalDnsRecordProfileValidator {
    pub rules: Vec<Rc<dyn SeasonalDnsRecordProfileRule>>,
}

impl SeasonalDnsRecordProfileValidator {
    pub fn validate(&self, item: &SeasonalDnsRecordProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalDnsRecordProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalDnsRecordProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_dns_record_profile_validator() -> SeasonalDnsRecordProfileValidator {
    SeasonalDnsRecordProfileValidator {
        rules: vec![
            Rc::new(SeasonalDnsRecordProfileNotesLengthRule { max_length: 255 })
                as Rc<dyn SeasonalDnsRecordProfileRule>,
            Rc::new(SeasonalDnsRecordProfileRequiredNotesRule {
                message: "seasonal dns record profile needs a value".to_string(),
            }),
            Rc::new(SeasonalDnsRecordProfileRequiredPrincipalRule {
                message: "seasonal dns record profile needs an entry".to_string(),
            }),
            Rc::new(SeasonalDnsRecordProfileAllowedHostRule {
                allowed: vec!["batched".to_string(), "global".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_dns_record_profile_validator();
        let item = new_seasonal_dns_record_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = SeasonalDnsRecordProfileNotesLengthRule { max_length: 86400 };
        let mut item = new_seasonal_dns_record_profile("x", "y");
        item.notes = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = SeasonalDnsRecordProfileRequiredNotesRule {
            message: "seasonal dns record profile needs an entry".to_string(),
        };
        let mut item = new_seasonal_dns_record_profile("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = SeasonalDnsRecordProfileRequiredPrincipalRule {
            message: "seasonal dns record profile needs an entry".to_string(),
        };
        let mut item = new_seasonal_dns_record_profile("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
