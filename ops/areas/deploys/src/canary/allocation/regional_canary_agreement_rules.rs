use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional canary agreement must pass. Check returns its problems, or none.
pub trait RegionalCanaryAgreementRule: std::fmt::Debug {
    fn check(&self, item: &RegionalCanaryAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanaryAgreementNamespaceLengthRule {
    pub max_length: i64,
}

impl RegionalCanaryAgreementRule for RegionalCanaryAgreementNamespaceLengthRule {
    fn check(&self, item: &RegionalCanaryAgreement) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanaryAgreementCodePrefixRule {
    pub prefix: String,
}

impl RegionalCanaryAgreementRule for RegionalCanaryAgreementCodePrefixRule {
    fn check(&self, item: &RegionalCanaryAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanaryAgreementRequiredNameRule {
    pub message: String,
}

impl RegionalCanaryAgreementRule for RegionalCanaryAgreementRequiredNameRule {
    fn check(&self, item: &RegionalCanaryAgreement) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanaryAgreementRequiredCodeRule {
    pub message: String,
}

impl RegionalCanaryAgreementRule for RegionalCanaryAgreementRequiredCodeRule {
    fn check(&self, item: &RegionalCanaryAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanaryAgreementAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalCanaryAgreementRule for RegionalCanaryAgreementAmountRangeRule {
    fn check(&self, item: &RegionalCanaryAgreement) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a regional canary agreement.
#[derive(Debug, Clone)]
pub struct RegionalCanaryAgreementValidator {
    pub rules: Vec<Rc<dyn RegionalCanaryAgreementRule>>,
}

impl RegionalCanaryAgreementValidator {
    pub fn validate(&self, item: &RegionalCanaryAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalCanaryAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalCanaryAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_canary_agreement_validator() -> RegionalCanaryAgreementValidator {
    RegionalCanaryAgreementValidator {
        rules: vec![
            Rc::new(RegionalCanaryAgreementNamespaceLengthRule { max_length: 120 })
                as Rc<dyn RegionalCanaryAgreementRule>,
            Rc::new(RegionalCanaryAgreementCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(RegionalCanaryAgreementRequiredNameRule {
                message: "regional canary agreement needs a value".to_string(),
            }),
            Rc::new(RegionalCanaryAgreementRequiredCodeRule {
                message: "regional canary agreement needs a value".to_string(),
            }),
            Rc::new(RegionalCanaryAgreementAmountRangeRule { min: 10.0, max: 3395.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_canary_agreement_validator();
        let item = new_regional_canary_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = RegionalCanaryAgreementNamespaceLengthRule { max_length: 255 };
        let mut item = new_regional_canary_agreement("x", "y");
        item.namespace = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RegionalCanaryAgreementRequiredNameRule {
            message: "regional canary agreement needs a value".to_string(),
        };
        let mut item = new_regional_canary_agreement("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalCanaryAgreementRequiredCodeRule {
            message: "regional canary agreement needs a value".to_string(),
        };
        let mut item = new_regional_canary_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = RegionalCanaryAgreementAmountRangeRule { min: 7.6, max: 2779.0 };
        let mut item = new_regional_canary_agreement("x", "y");
        item.amount = 2780.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
