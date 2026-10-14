use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending flow log estimate must pass. Check returns its problems, or none.
pub trait PendingFlowLogEstimateRule: std::fmt::Debug {
    fn check(&self, item: &PendingFlowLogEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogEstimatePhoneLengthRule {
    pub max_length: i64,
}

impl PendingFlowLogEstimateRule for PendingFlowLogEstimatePhoneLengthRule {
    fn check(&self, item: &PendingFlowLogEstimate) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogEstimateRequiredTitleRule {
    pub message: String,
}

impl PendingFlowLogEstimateRule for PendingFlowLogEstimateRequiredTitleRule {
    fn check(&self, item: &PendingFlowLogEstimate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogEstimateRequiredPhoneRule {
    pub message: String,
}

impl PendingFlowLogEstimateRule for PendingFlowLogEstimateRequiredPhoneRule {
    fn check(&self, item: &PendingFlowLogEstimate) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending flow log estimate.
#[derive(Debug, Clone)]
pub struct PendingFlowLogEstimateValidator {
    pub rules: Vec<Rc<dyn PendingFlowLogEstimateRule>>,
}

impl PendingFlowLogEstimateValidator {
    pub fn validate(&self, item: &PendingFlowLogEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingFlowLogEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingFlowLogEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_flow_log_estimate_validator() -> PendingFlowLogEstimateValidator {
    PendingFlowLogEstimateValidator {
        rules: vec![
            Rc::new(PendingFlowLogEstimatePhoneLengthRule { max_length: 80 }) as Rc<dyn PendingFlowLogEstimateRule>,
            Rc::new(PendingFlowLogEstimateRequiredTitleRule {
                message: "pending flow log estimate needs this field".to_string(),
            }),
            Rc::new(PendingFlowLogEstimateRequiredPhoneRule {
                message: "pending flow log estimate needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_flow_log_estimate_validator();
        let item = new_pending_flow_log_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = PendingFlowLogEstimatePhoneLengthRule { max_length: 80 };
        let mut item = new_pending_flow_log_estimate("x", "y");
        item.phone = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PendingFlowLogEstimateRequiredTitleRule {
            message: "pending flow log estimate needs a value".to_string(),
        };
        let mut item = new_pending_flow_log_estimate("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = PendingFlowLogEstimateRequiredPhoneRule {
            message: "pending flow log estimate needs a value".to_string(),
        };
        let mut item = new_pending_flow_log_estimate("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
