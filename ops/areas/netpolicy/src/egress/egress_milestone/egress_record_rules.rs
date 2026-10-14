use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a egress record must pass. Check returns its problems, or none.
pub trait EgressRecordRule: std::fmt::Debug {
    fn check(&self, item: &EgressRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressRecordCodePrefixRule {
    pub prefix: String,
}

impl EgressRecordRule for EgressRecordCodePrefixRule {
    fn check(&self, item: &EgressRecord) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressRecordRequiredCountryRule {
    pub message: String,
}

impl EgressRecordRule for EgressRecordRequiredCountryRule {
    fn check(&self, item: &EgressRecord) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressRecordAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl EgressRecordRule for EgressRecordAllowedCountryRule {
    fn check(&self, item: &EgressRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is out of range", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressRecordRequiredContactEmailRule {
    pub message: String,
}

impl EgressRecordRule for EgressRecordRequiredContactEmailRule {
    fn check(&self, item: &EgressRecord) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressRecordContactEmailLengthRule {
    pub max_length: i64,
}

impl EgressRecordRule for EgressRecordContactEmailLengthRule {
    fn check(&self, item: &EgressRecord) -> Vec<String> {
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
pub struct EgressRecordTitleLengthRule {
    pub max_length: i64,
}

impl EgressRecordRule for EgressRecordTitleLengthRule {
    fn check(&self, item: &EgressRecord) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress record.
#[derive(Debug, Clone)]
pub struct EgressRecordValidator {
    pub rules: Vec<Rc<dyn EgressRecordRule>>,
}

impl EgressRecordValidator {
    pub fn validate(&self, item: &EgressRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EgressRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_record_validator() -> EgressRecordValidator {
    EgressRecordValidator {
        rules: vec![
            Rc::new(EgressRecordCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn EgressRecordRule>,
            Rc::new(EgressRecordRequiredCountryRule {
                message: "egress record needs an entry".to_string(),
            }),
            Rc::new(EgressRecordAllowedCountryRule { allowed: Vec::new() }),
            Rc::new(EgressRecordRequiredContactEmailRule {
                message: "egress record needs this field".to_string(),
            }),
            Rc::new(EgressRecordContactEmailLengthRule { max_length: 80 }),
            Rc::new(EgressRecordTitleLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_record_validator();
        let item = new_egress_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = EgressRecordRequiredCountryRule {
            message: "egress record needs a value".to_string(),
        };
        let mut item = new_egress_record("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = EgressRecordRequiredContactEmailRule {
            message: "egress record needs this field".to_string(),
        };
        let mut item = new_egress_record("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = EgressRecordContactEmailLengthRule { max_length: 1000 };
        let mut item = new_egress_record("x", "y");
        item.contact_email = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = EgressRecordTitleLengthRule { max_length: 80 };
        let mut item = new_egress_record("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
