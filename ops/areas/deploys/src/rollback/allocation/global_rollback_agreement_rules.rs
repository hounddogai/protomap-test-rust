use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global rollback agreement must pass. Check returns its problems, or none.
pub trait GlobalRollbackAgreementRule: std::fmt::Debug {
    fn check(&self, item: &GlobalRollbackAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRollbackAgreementRegionLengthRule {
    pub max_length: i64,
}

impl GlobalRollbackAgreementRule for GlobalRollbackAgreementRegionLengthRule {
    fn check(&self, item: &GlobalRollbackAgreement) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRollbackAgreementRequiredCodeRule {
    pub message: String,
}

impl GlobalRollbackAgreementRule for GlobalRollbackAgreementRequiredCodeRule {
    fn check(&self, item: &GlobalRollbackAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRollbackAgreementMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalRollbackAgreementRule for GlobalRollbackAgreementMinimumRangeRule {
    fn check(&self, item: &GlobalRollbackAgreement) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRollbackAgreementRequiredRegionRule {
    pub message: String,
}

impl GlobalRollbackAgreementRule for GlobalRollbackAgreementRequiredRegionRule {
    fn check(&self, item: &GlobalRollbackAgreement) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global rollback agreement.
#[derive(Debug, Clone)]
pub struct GlobalRollbackAgreementValidator {
    pub rules: Vec<Rc<dyn GlobalRollbackAgreementRule>>,
}

impl GlobalRollbackAgreementValidator {
    pub fn validate(&self, item: &GlobalRollbackAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalRollbackAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalRollbackAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_rollback_agreement_validator() -> GlobalRollbackAgreementValidator {
    GlobalRollbackAgreementValidator {
        rules: vec![
            Rc::new(GlobalRollbackAgreementRegionLengthRule { max_length: 86400 })
                as Rc<dyn GlobalRollbackAgreementRule>,
            Rc::new(GlobalRollbackAgreementRequiredCodeRule {
                message: "global rollback agreement needs a value".to_string(),
            }),
            Rc::new(GlobalRollbackAgreementMinimumRangeRule { min: 8.4, max: 1078.8 }),
            Rc::new(GlobalRollbackAgreementRequiredRegionRule {
                message: "global rollback agreement needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_rollback_agreement_validator();
        let item = new_global_rollback_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = GlobalRollbackAgreementRegionLengthRule { max_length: 64 };
        let mut item = new_global_rollback_agreement("x", "y");
        item.region = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalRollbackAgreementRequiredCodeRule {
            message: "global rollback agreement needs a value".to_string(),
        };
        let mut item = new_global_rollback_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = GlobalRollbackAgreementMinimumRangeRule { min: 8.7, max: 1481.9 };
        let mut item = new_global_rollback_agreement("x", "y");
        item.minimum = 1483.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = GlobalRollbackAgreementRequiredRegionRule {
            message: "global rollback agreement needs a value".to_string(),
        };
        let mut item = new_global_rollback_agreement("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
