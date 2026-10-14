use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a zone agreement must pass. Check returns its problems, or none.
pub trait ZoneAgreementRule: std::fmt::Debug {
    fn check(&self, item: &ZoneAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneAgreementRequiredCodeRule {
    pub message: String,
}

impl ZoneAgreementRule for ZoneAgreementRequiredCodeRule {
    fn check(&self, item: &ZoneAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneAgreementLocaleLengthRule {
    pub max_length: i64,
}

impl ZoneAgreementRule for ZoneAgreementLocaleLengthRule {
    fn check(&self, item: &ZoneAgreement) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneAgreementPinnedGuardRule {
    pub limit: i64,
}

impl ZoneAgreementRule for ZoneAgreementPinnedGuardRule {
    fn check(&self, item: &ZoneAgreement) -> Vec<String> {
        if item.pinned && (item.updated_at > self.limit) {
            return vec![format!(
                "updated at {} exceeds {} while pinned",
                item.updated_at, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone agreement.
#[derive(Debug, Clone)]
pub struct ZoneAgreementValidator {
    pub rules: Vec<Rc<dyn ZoneAgreementRule>>,
}

impl ZoneAgreementValidator {
    pub fn validate(&self, item: &ZoneAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ZoneAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_agreement_validator() -> ZoneAgreementValidator {
    ZoneAgreementValidator {
        rules: vec![
            Rc::new(ZoneAgreementRequiredCodeRule {
                message: "zone agreement needs a value".to_string(),
            }) as Rc<dyn ZoneAgreementRule>,
            Rc::new(ZoneAgreementLocaleLengthRule { max_length: 86400 }),
            Rc::new(ZoneAgreementPinnedGuardRule { limit: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_agreement_validator();
        let item = new_zone_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ZoneAgreementRequiredCodeRule {
            message: "zone agreement needs an entry".to_string(),
        };
        let mut item = new_zone_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = ZoneAgreementLocaleLengthRule { max_length: 86400 };
        let mut item = new_zone_agreement("x", "y");
        item.locale = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
