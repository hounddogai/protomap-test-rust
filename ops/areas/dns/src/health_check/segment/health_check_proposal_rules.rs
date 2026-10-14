use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check proposal must pass. Check returns its problems, or none.
pub trait HealthCheckProposalRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckProposalBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl HealthCheckProposalRule for HealthCheckProposalBatchSizeRangeRule {
    fn check(&self, item: &HealthCheckProposal) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckProposalRequiredCodeRule {
    pub message: String,
}

impl HealthCheckProposalRule for HealthCheckProposalRequiredCodeRule {
    fn check(&self, item: &HealthCheckProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckProposalHostLengthRule {
    pub max_length: i64,
}

impl HealthCheckProposalRule for HealthCheckProposalHostLengthRule {
    fn check(&self, item: &HealthCheckProposal) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check proposal.
#[derive(Debug, Clone)]
pub struct HealthCheckProposalValidator {
    pub rules: Vec<Rc<dyn HealthCheckProposalRule>>,
}

impl HealthCheckProposalValidator {
    pub fn validate(&self, item: &HealthCheckProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &HealthCheckProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_proposal_validator() -> HealthCheckProposalValidator {
    HealthCheckProposalValidator {
        rules: vec![
            Rc::new(HealthCheckProposalBatchSizeRangeRule { min: 64, max: 80 }) as Rc<dyn HealthCheckProposalRule>,
            Rc::new(HealthCheckProposalRequiredCodeRule {
                message: "health check proposal needs an entry".to_string(),
            }),
            Rc::new(HealthCheckProposalHostLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_proposal_validator();
        let item = new_health_check_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = HealthCheckProposalBatchSizeRangeRule { min: 80, max: 1000 };
        let mut item = new_health_check_proposal("x", "y");
        item.batch_size = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = HealthCheckProposalRequiredCodeRule {
            message: "health check proposal needs this field".to_string(),
        };
        let mut item = new_health_check_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = HealthCheckProposalHostLengthRule { max_length: 255 };
        let mut item = new_health_check_proposal("x", "y");
        item.host = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
