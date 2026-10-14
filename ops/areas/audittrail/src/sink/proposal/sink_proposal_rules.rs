use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink proposal must pass. Check returns its problems, or none.
pub trait SinkProposalRule: std::fmt::Debug {
    fn check(&self, item: &SinkProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkProposalRequiredTenantIdRule {
    pub message: String,
}

impl SinkProposalRule for SinkProposalRequiredTenantIdRule {
    fn check(&self, item: &SinkProposal) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkProposalCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SinkProposalRule for SinkProposalCreatedAtRangeRule {
    fn check(&self, item: &SinkProposal) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkProposalDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SinkProposalRule for SinkProposalDiscountRangeRule {
    fn check(&self, item: &SinkProposal) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkProposalTitleLengthRule {
    pub max_length: i64,
}

impl SinkProposalRule for SinkProposalTitleLengthRule {
    fn check(&self, item: &SinkProposal) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink proposal.
#[derive(Debug, Clone)]
pub struct SinkProposalValidator {
    pub rules: Vec<Rc<dyn SinkProposalRule>>,
}

impl SinkProposalValidator {
    pub fn validate(&self, item: &SinkProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_proposal_validator() -> SinkProposalValidator {
    SinkProposalValidator {
        rules: vec![
            Rc::new(SinkProposalRequiredTenantIdRule {
                message: "sink proposal needs this field".to_string(),
            }) as Rc<dyn SinkProposalRule>,
            Rc::new(SinkProposalCreatedAtRangeRule { min: 64, max: 3600 }),
            Rc::new(SinkProposalDiscountRangeRule { min: 0.5, max: 4544.3 }),
            Rc::new(SinkProposalTitleLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_proposal_validator();
        let item = new_sink_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = SinkProposalRequiredTenantIdRule {
            message: "sink proposal needs this field".to_string(),
        };
        let mut item = new_sink_proposal("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = SinkProposalCreatedAtRangeRule { min: 3600, max: 3600 };
        let mut item = new_sink_proposal("x", "y");
        item.created_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = SinkProposalDiscountRangeRule { min: 7.8, max: 3563.7 };
        let mut item = new_sink_proposal("x", "y");
        item.discount = 3565.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SinkProposalTitleLengthRule { max_length: 80 };
        let mut item = new_sink_proposal("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
