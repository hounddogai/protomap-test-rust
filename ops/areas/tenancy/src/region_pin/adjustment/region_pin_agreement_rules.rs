use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin agreement must pass. Check returns its problems, or none.
pub trait RegionPinAgreementRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinAgreementRequiredTitleRule {
    pub message: String,
}

impl RegionPinAgreementRule for RegionPinAgreementRequiredTitleRule {
    fn check(&self, item: &RegionPinAgreement) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinAgreementRequiredChecksumRule {
    pub message: String,
}

impl RegionPinAgreementRule for RegionPinAgreementRequiredChecksumRule {
    fn check(&self, item: &RegionPinAgreement) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinAgreementChecksumLengthRule {
    pub max_length: i64,
}

impl RegionPinAgreementRule for RegionPinAgreementChecksumLengthRule {
    fn check(&self, item: &RegionPinAgreement) -> Vec<String> {
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
pub struct RegionPinAgreementCodeLengthRule {
    pub max_length: i64,
}

impl RegionPinAgreementRule for RegionPinAgreementCodeLengthRule {
    fn check(&self, item: &RegionPinAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinAgreementRequiredCodeRule {
    pub message: String,
}

impl RegionPinAgreementRule for RegionPinAgreementRequiredCodeRule {
    fn check(&self, item: &RegionPinAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinAgreementRequiredPrincipalRule {
    pub message: String,
}

impl RegionPinAgreementRule for RegionPinAgreementRequiredPrincipalRule {
    fn check(&self, item: &RegionPinAgreement) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin agreement.
#[derive(Debug, Clone)]
pub struct RegionPinAgreementValidator {
    pub rules: Vec<Rc<dyn RegionPinAgreementRule>>,
}

impl RegionPinAgreementValidator {
    pub fn validate(&self, item: &RegionPinAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionPinAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_agreement_validator() -> RegionPinAgreementValidator {
    RegionPinAgreementValidator {
        rules: vec![
            Rc::new(RegionPinAgreementRequiredTitleRule {
                message: "region pin agreement needs a value".to_string(),
            }) as Rc<dyn RegionPinAgreementRule>,
            Rc::new(RegionPinAgreementRequiredChecksumRule {
                message: "region pin agreement needs a value".to_string(),
            }),
            Rc::new(RegionPinAgreementChecksumLengthRule { max_length: 64 }),
            Rc::new(RegionPinAgreementCodeLengthRule { max_length: 80 }),
            Rc::new(RegionPinAgreementRequiredCodeRule {
                message: "region pin agreement needs a value".to_string(),
            }),
            Rc::new(RegionPinAgreementRequiredPrincipalRule {
                message: "region pin agreement needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_agreement_validator();
        let item = new_region_pin_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RegionPinAgreementRequiredTitleRule {
            message: "region pin agreement needs a value".to_string(),
        };
        let mut item = new_region_pin_agreement("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = RegionPinAgreementRequiredChecksumRule {
            message: "region pin agreement needs an entry".to_string(),
        };
        let mut item = new_region_pin_agreement("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = RegionPinAgreementChecksumLengthRule { max_length: 255 };
        let mut item = new_region_pin_agreement("x", "y");
        item.checksum = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionPinAgreementCodeLengthRule { max_length: 32 };
        let mut item = new_region_pin_agreement("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionPinAgreementRequiredCodeRule {
            message: "region pin agreement needs this field".to_string(),
        };
        let mut item = new_region_pin_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = RegionPinAgreementRequiredPrincipalRule {
            message: "region pin agreement needs this field".to_string(),
        };
        let mut item = new_region_pin_agreement("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
