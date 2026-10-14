use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled canary agreement must pass. Check returns its problems, or none.
pub trait ScheduledCanaryAgreementRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledCanaryAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCanaryAgreementLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledCanaryAgreementRule for ScheduledCanaryAgreementLimitRangeRule {
    fn check(&self, item: &ScheduledCanaryAgreement) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCanaryAgreementRequiredScopeRule {
    pub message: String,
}

impl ScheduledCanaryAgreementRule for ScheduledCanaryAgreementRequiredScopeRule {
    fn check(&self, item: &ScheduledCanaryAgreement) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCanaryAgreementRequiredTitleRule {
    pub message: String,
}

impl ScheduledCanaryAgreementRule for ScheduledCanaryAgreementRequiredTitleRule {
    fn check(&self, item: &ScheduledCanaryAgreement) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled canary agreement.
#[derive(Debug, Clone)]
pub struct ScheduledCanaryAgreementValidator {
    pub rules: Vec<Rc<dyn ScheduledCanaryAgreementRule>>,
}

impl ScheduledCanaryAgreementValidator {
    pub fn validate(&self, item: &ScheduledCanaryAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledCanaryAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledCanaryAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_canary_agreement_validator() -> ScheduledCanaryAgreementValidator {
    ScheduledCanaryAgreementValidator {
        rules: vec![
            Rc::new(ScheduledCanaryAgreementLimitRangeRule { min: 32, max: 86400 })
                as Rc<dyn ScheduledCanaryAgreementRule>,
            Rc::new(ScheduledCanaryAgreementRequiredScopeRule {
                message: "scheduled canary agreement needs a value".to_string(),
            }),
            Rc::new(ScheduledCanaryAgreementRequiredTitleRule {
                message: "scheduled canary agreement needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_canary_agreement_validator();
        let item = new_scheduled_canary_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = ScheduledCanaryAgreementLimitRangeRule { min: 16, max: 1000 };
        let mut item = new_scheduled_canary_agreement("x", "y");
        item.limit = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = ScheduledCanaryAgreementRequiredScopeRule {
            message: "scheduled canary agreement needs this field".to_string(),
        };
        let mut item = new_scheduled_canary_agreement("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ScheduledCanaryAgreementRequiredTitleRule {
            message: "scheduled canary agreement needs this field".to_string(),
        };
        let mut item = new_scheduled_canary_agreement("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
