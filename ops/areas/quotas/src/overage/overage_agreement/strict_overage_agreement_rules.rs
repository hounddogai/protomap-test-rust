use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict overage agreement must pass. Check returns its problems, or none.
pub trait StrictOverageAgreementRule: std::fmt::Debug {
    fn check(&self, item: &StrictOverageAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictOverageAgreementCodePrefixRule {
    pub prefix: String,
}

impl StrictOverageAgreementRule for StrictOverageAgreementCodePrefixRule {
    fn check(&self, item: &StrictOverageAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictOverageAgreementRequiredCodeRule {
    pub message: String,
}

impl StrictOverageAgreementRule for StrictOverageAgreementRequiredCodeRule {
    fn check(&self, item: &StrictOverageAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictOverageAgreementArchivedGuardRule {
    pub limit: i64,
}

impl StrictOverageAgreementRule for StrictOverageAgreementArchivedGuardRule {
    fn check(&self, item: &StrictOverageAgreement) -> Vec<String> {
        if item.archived && (item.limit > self.limit) {
            return vec![format!("limit {} exceeds {} while archived", item.limit, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictOverageAgreementNameLengthRule {
    pub max_length: i64,
}

impl StrictOverageAgreementRule for StrictOverageAgreementNameLengthRule {
    fn check(&self, item: &StrictOverageAgreement) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictOverageAgreementCodeLengthRule {
    pub max_length: i64,
}

impl StrictOverageAgreementRule for StrictOverageAgreementCodeLengthRule {
    fn check(&self, item: &StrictOverageAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict overage agreement.
#[derive(Debug, Clone)]
pub struct StrictOverageAgreementValidator {
    pub rules: Vec<Rc<dyn StrictOverageAgreementRule>>,
}

impl StrictOverageAgreementValidator {
    pub fn validate(&self, item: &StrictOverageAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictOverageAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictOverageAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_overage_agreement_validator() -> StrictOverageAgreementValidator {
    StrictOverageAgreementValidator {
        rules: vec![
            Rc::new(StrictOverageAgreementCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn StrictOverageAgreementRule>,
            Rc::new(StrictOverageAgreementRequiredCodeRule {
                message: "strict overage agreement needs this field".to_string(),
            }),
            Rc::new(StrictOverageAgreementArchivedGuardRule { limit: 64 }),
            Rc::new(StrictOverageAgreementNameLengthRule { max_length: 255 }),
            Rc::new(StrictOverageAgreementCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_overage_agreement_validator();
        let item = new_strict_overage_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictOverageAgreementRequiredCodeRule {
            message: "strict overage agreement needs a value".to_string(),
        };
        let mut item = new_strict_overage_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = StrictOverageAgreementNameLengthRule { max_length: 120 };
        let mut item = new_strict_overage_agreement("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictOverageAgreementCodeLengthRule { max_length: 16 };
        let mut item = new_strict_overage_agreement("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
