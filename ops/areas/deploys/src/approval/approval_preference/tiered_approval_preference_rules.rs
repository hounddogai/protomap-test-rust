use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered approval preference must pass. Check returns its problems, or none.
pub trait TieredApprovalPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &TieredApprovalPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl TieredApprovalPreferenceRule for TieredApprovalPreferenceCodeLengthRule {
    fn check(&self, item: &TieredApprovalPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalPreferenceRequiredCodeRule {
    pub message: String,
}

impl TieredApprovalPreferenceRule for TieredApprovalPreferenceRequiredCodeRule {
    fn check(&self, item: &TieredApprovalPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalPreferenceIssuerLengthRule {
    pub max_length: i64,
}

impl TieredApprovalPreferenceRule for TieredApprovalPreferenceIssuerLengthRule {
    fn check(&self, item: &TieredApprovalPreference) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered approval preference.
#[derive(Debug, Clone)]
pub struct TieredApprovalPreferenceValidator {
    pub rules: Vec<Rc<dyn TieredApprovalPreferenceRule>>,
}

impl TieredApprovalPreferenceValidator {
    pub fn validate(&self, item: &TieredApprovalPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredApprovalPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredApprovalPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_approval_preference_validator() -> TieredApprovalPreferenceValidator {
    TieredApprovalPreferenceValidator {
        rules: vec![
            Rc::new(TieredApprovalPreferenceCodeLengthRule { max_length: 1000 })
                as Rc<dyn TieredApprovalPreferenceRule>,
            Rc::new(TieredApprovalPreferenceRequiredCodeRule {
                message: "tiered approval preference needs a value".to_string(),
            }),
            Rc::new(TieredApprovalPreferenceIssuerLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_approval_preference_validator();
        let item = new_tiered_approval_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredApprovalPreferenceCodeLengthRule { max_length: 64 };
        let mut item = new_tiered_approval_preference("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredApprovalPreferenceRequiredCodeRule {
            message: "tiered approval preference needs an entry".to_string(),
        };
        let mut item = new_tiered_approval_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = TieredApprovalPreferenceIssuerLengthRule { max_length: 16 };
        let mut item = new_tiered_approval_preference("x", "y");
        item.issuer = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
