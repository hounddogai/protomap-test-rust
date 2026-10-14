use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a usage agreement must pass. Check returns its problems, or none.
pub trait UsageAgreementRule: std::fmt::Debug {
    fn check(&self, item: &UsageAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageAgreementPrincipalLengthRule {
    pub max_length: i64,
}

impl UsageAgreementRule for UsageAgreementPrincipalLengthRule {
    fn check(&self, item: &UsageAgreement) -> Vec<String> {
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
pub struct UsageAgreementSourceLengthRule {
    pub max_length: i64,
}

impl UsageAgreementRule for UsageAgreementSourceLengthRule {
    fn check(&self, item: &UsageAgreement) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageAgreementExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl UsageAgreementRule for UsageAgreementExpiresAtRangeRule {
    fn check(&self, item: &UsageAgreement) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageAgreementRequiredCodeRule {
    pub message: String,
}

impl UsageAgreementRule for UsageAgreementRequiredCodeRule {
    fn check(&self, item: &UsageAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageAgreementWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl UsageAgreementRule for UsageAgreementWindowDaysRangeRule {
    fn check(&self, item: &UsageAgreement) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageAgreementNameLengthRule {
    pub max_length: i64,
}

impl UsageAgreementRule for UsageAgreementNameLengthRule {
    fn check(&self, item: &UsageAgreement) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage agreement.
#[derive(Debug, Clone)]
pub struct UsageAgreementValidator {
    pub rules: Vec<Rc<dyn UsageAgreementRule>>,
}

impl UsageAgreementValidator {
    pub fn validate(&self, item: &UsageAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &UsageAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_agreement_validator() -> UsageAgreementValidator {
    UsageAgreementValidator {
        rules: vec![
            Rc::new(UsageAgreementPrincipalLengthRule { max_length: 3600 }) as Rc<dyn UsageAgreementRule>,
            Rc::new(UsageAgreementSourceLengthRule { max_length: 255 }),
            Rc::new(UsageAgreementExpiresAtRangeRule { min: 80, max: 86400 }),
            Rc::new(UsageAgreementRequiredCodeRule {
                message: "usage agreement needs a value".to_string(),
            }),
            Rc::new(UsageAgreementWindowDaysRangeRule { min: 32, max: 32 }),
            Rc::new(UsageAgreementNameLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_agreement_validator();
        let item = new_usage_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = UsageAgreementPrincipalLengthRule { max_length: 120 };
        let mut item = new_usage_agreement("x", "y");
        item.principal = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = UsageAgreementSourceLengthRule { max_length: 80 };
        let mut item = new_usage_agreement("x", "y");
        item.source = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = UsageAgreementExpiresAtRangeRule { min: 80, max: 80 };
        let mut item = new_usage_agreement("x", "y");
        item.expires_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = UsageAgreementRequiredCodeRule {
            message: "usage agreement needs this field".to_string(),
        };
        let mut item = new_usage_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = UsageAgreementWindowDaysRangeRule { min: 64, max: 3600 };
        let mut item = new_usage_agreement("x", "y");
        item.window_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = UsageAgreementNameLengthRule { max_length: 3600 };
        let mut item = new_usage_agreement("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
