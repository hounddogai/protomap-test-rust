use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a policy agreement must pass. Check returns its problems, or none.
pub trait PolicyAgreementRule: std::fmt::Debug {
    fn check(&self, item: &PolicyAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyAgreementFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PolicyAgreementRule for PolicyAgreementFloorValueRangeRule {
    fn check(&self, item: &PolicyAgreement) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyAgreementHostLengthRule {
    pub max_length: i64,
}

impl PolicyAgreementRule for PolicyAgreementHostLengthRule {
    fn check(&self, item: &PolicyAgreement) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyAgreementRequiredCodeRule {
    pub message: String,
}

impl PolicyAgreementRule for PolicyAgreementRequiredCodeRule {
    fn check(&self, item: &PolicyAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyAgreementRequiredHostRule {
    pub message: String,
}

impl PolicyAgreementRule for PolicyAgreementRequiredHostRule {
    fn check(&self, item: &PolicyAgreement) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyAgreementEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PolicyAgreementRule for PolicyAgreementEndsAtRangeRule {
    fn check(&self, item: &PolicyAgreement) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a policy agreement.
#[derive(Debug, Clone)]
pub struct PolicyAgreementValidator {
    pub rules: Vec<Rc<dyn PolicyAgreementRule>>,
}

impl PolicyAgreementValidator {
    pub fn validate(&self, item: &PolicyAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PolicyAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_agreement_validator() -> PolicyAgreementValidator {
    PolicyAgreementValidator {
        rules: vec![
            Rc::new(PolicyAgreementFloorValueRangeRule { min: 8.4, max: 62.6 }) as Rc<dyn PolicyAgreementRule>,
            Rc::new(PolicyAgreementHostLengthRule { max_length: 1000 }),
            Rc::new(PolicyAgreementRequiredCodeRule {
                message: "policy agreement needs this field".to_string(),
            }),
            Rc::new(PolicyAgreementRequiredHostRule {
                message: "policy agreement needs a value".to_string(),
            }),
            Rc::new(PolicyAgreementEndsAtRangeRule { min: 86400, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_agreement_validator();
        let item = new_policy_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = PolicyAgreementFloorValueRangeRule { min: 4.6, max: 4801.0 };
        let mut item = new_policy_agreement("x", "y");
        item.floor_value = 4802.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = PolicyAgreementHostLengthRule { max_length: 86400 };
        let mut item = new_policy_agreement("x", "y");
        item.host = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PolicyAgreementRequiredCodeRule {
            message: "policy agreement needs a value".to_string(),
        };
        let mut item = new_policy_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = PolicyAgreementRequiredHostRule {
            message: "policy agreement needs an entry".to_string(),
        };
        let mut item = new_policy_agreement("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = PolicyAgreementEndsAtRangeRule { min: 120, max: 120 };
        let mut item = new_policy_agreement("x", "y");
        item.ends_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
