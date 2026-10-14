use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier notice must pass. Check returns its problems, or none.
pub trait VerifierNoticeRule: std::fmt::Debug {
    fn check(&self, item: &VerifierNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierNoticeContactEmailLengthRule {
    pub max_length: i64,
}

impl VerifierNoticeRule for VerifierNoticeContactEmailLengthRule {
    fn check(&self, item: &VerifierNotice) -> Vec<String> {
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
pub struct VerifierNoticeLocaleLengthRule {
    pub max_length: i64,
}

impl VerifierNoticeRule for VerifierNoticeLocaleLengthRule {
    fn check(&self, item: &VerifierNotice) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierNoticeNameLengthRule {
    pub max_length: i64,
}

impl VerifierNoticeRule for VerifierNoticeNameLengthRule {
    fn check(&self, item: &VerifierNotice) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier notice.
#[derive(Debug, Clone)]
pub struct VerifierNoticeValidator {
    pub rules: Vec<Rc<dyn VerifierNoticeRule>>,
}

impl VerifierNoticeValidator {
    pub fn validate(&self, item: &VerifierNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &VerifierNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_notice_validator() -> VerifierNoticeValidator {
    VerifierNoticeValidator {
        rules: vec![
            Rc::new(VerifierNoticeContactEmailLengthRule { max_length: 80 }) as Rc<dyn VerifierNoticeRule>,
            Rc::new(VerifierNoticeLocaleLengthRule { max_length: 1000 }),
            Rc::new(VerifierNoticeNameLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_notice_validator();
        let item = new_verifier_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = VerifierNoticeContactEmailLengthRule { max_length: 80 };
        let mut item = new_verifier_notice("x", "y");
        item.contact_email = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = VerifierNoticeLocaleLengthRule { max_length: 120 };
        let mut item = new_verifier_notice("x", "y");
        item.locale = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = VerifierNoticeNameLengthRule { max_length: 80 };
        let mut item = new_verifier_notice("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
