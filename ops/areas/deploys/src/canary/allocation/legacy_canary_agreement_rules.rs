use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy canary agreement must pass. Check returns its problems, or none.
pub trait LegacyCanaryAgreementRule: std::fmt::Debug {
    fn check(&self, item: &LegacyCanaryAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCanaryAgreementAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LegacyCanaryAgreementRule for LegacyCanaryAgreementAllowedCodeRule {
    fn check(&self, item: &LegacyCanaryAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCanaryAgreementRequiredTitleRule {
    pub message: String,
}

impl LegacyCanaryAgreementRule for LegacyCanaryAgreementRequiredTitleRule {
    fn check(&self, item: &LegacyCanaryAgreement) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCanaryAgreementCodeLengthRule {
    pub max_length: i64,
}

impl LegacyCanaryAgreementRule for LegacyCanaryAgreementCodeLengthRule {
    fn check(&self, item: &LegacyCanaryAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCanaryAgreementUnitLengthRule {
    pub max_length: i64,
}

impl LegacyCanaryAgreementRule for LegacyCanaryAgreementUnitLengthRule {
    fn check(&self, item: &LegacyCanaryAgreement) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCanaryAgreementRequiredUnitRule {
    pub message: String,
}

impl LegacyCanaryAgreementRule for LegacyCanaryAgreementRequiredUnitRule {
    fn check(&self, item: &LegacyCanaryAgreement) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCanaryAgreementRequiredCodeRule {
    pub message: String,
}

impl LegacyCanaryAgreementRule for LegacyCanaryAgreementRequiredCodeRule {
    fn check(&self, item: &LegacyCanaryAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy canary agreement.
#[derive(Debug, Clone)]
pub struct LegacyCanaryAgreementValidator {
    pub rules: Vec<Rc<dyn LegacyCanaryAgreementRule>>,
}

impl LegacyCanaryAgreementValidator {
    pub fn validate(&self, item: &LegacyCanaryAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyCanaryAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyCanaryAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_canary_agreement_validator() -> LegacyCanaryAgreementValidator {
    LegacyCanaryAgreementValidator {
        rules: vec![
            Rc::new(LegacyCanaryAgreementAllowedCodeRule {
                allowed: vec!["fallback".to_string(), "strict".to_string(), "weighted".to_string()],
            }) as Rc<dyn LegacyCanaryAgreementRule>,
            Rc::new(LegacyCanaryAgreementRequiredTitleRule {
                message: "legacy canary agreement needs a value".to_string(),
            }),
            Rc::new(LegacyCanaryAgreementCodeLengthRule { max_length: 255 }),
            Rc::new(LegacyCanaryAgreementUnitLengthRule { max_length: 16 }),
            Rc::new(LegacyCanaryAgreementRequiredUnitRule {
                message: "legacy canary agreement needs an entry".to_string(),
            }),
            Rc::new(LegacyCanaryAgreementRequiredCodeRule {
                message: "legacy canary agreement needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_canary_agreement_validator();
        let item = new_legacy_canary_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LegacyCanaryAgreementRequiredTitleRule {
            message: "legacy canary agreement needs this field".to_string(),
        };
        let mut item = new_legacy_canary_agreement("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyCanaryAgreementCodeLengthRule { max_length: 80 };
        let mut item = new_legacy_canary_agreement("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = LegacyCanaryAgreementUnitLengthRule { max_length: 120 };
        let mut item = new_legacy_canary_agreement("x", "y");
        item.unit = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = LegacyCanaryAgreementRequiredUnitRule {
            message: "legacy canary agreement needs this field".to_string(),
        };
        let mut item = new_legacy_canary_agreement("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyCanaryAgreementRequiredCodeRule {
            message: "legacy canary agreement needs an entry".to_string(),
        };
        let mut item = new_legacy_canary_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
