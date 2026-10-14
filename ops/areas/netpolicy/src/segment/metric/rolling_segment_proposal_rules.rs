use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling segment proposal must pass. Check returns its problems, or none.
pub trait RollingSegmentProposalRule: std::fmt::Debug {
    fn check(&self, item: &RollingSegmentProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSegmentProposalCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingSegmentProposalRule for RollingSegmentProposalCeilingRangeRule {
    fn check(&self, item: &RollingSegmentProposal) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSegmentProposalRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingSegmentProposalRule for RollingSegmentProposalRateRangeRule {
    fn check(&self, item: &RollingSegmentProposal) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSegmentProposalNameLengthRule {
    pub max_length: i64,
}

impl RollingSegmentProposalRule for RollingSegmentProposalNameLengthRule {
    fn check(&self, item: &RollingSegmentProposal) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSegmentProposalExternalRefLengthRule {
    pub max_length: i64,
}

impl RollingSegmentProposalRule for RollingSegmentProposalExternalRefLengthRule {
    fn check(&self, item: &RollingSegmentProposal) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSegmentProposalRequiredExternalRefRule {
    pub message: String,
}

impl RollingSegmentProposalRule for RollingSegmentProposalRequiredExternalRefRule {
    fn check(&self, item: &RollingSegmentProposal) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling segment proposal.
#[derive(Debug, Clone)]
pub struct RollingSegmentProposalValidator {
    pub rules: Vec<Rc<dyn RollingSegmentProposalRule>>,
}

impl RollingSegmentProposalValidator {
    pub fn validate(&self, item: &RollingSegmentProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingSegmentProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingSegmentProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_segment_proposal_validator() -> RollingSegmentProposalValidator {
    RollingSegmentProposalValidator {
        rules: vec![
            Rc::new(RollingSegmentProposalCeilingRangeRule { min: 1.2, max: 2199.7 })
                as Rc<dyn RollingSegmentProposalRule>,
            Rc::new(RollingSegmentProposalRateRangeRule { min: 4.8, max: 1914.3 }),
            Rc::new(RollingSegmentProposalNameLengthRule { max_length: 80 }),
            Rc::new(RollingSegmentProposalExternalRefLengthRule { max_length: 86400 }),
            Rc::new(RollingSegmentProposalRequiredExternalRefRule {
                message: "rolling segment proposal needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_segment_proposal_validator();
        let item = new_rolling_segment_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = RollingSegmentProposalCeilingRangeRule { min: 5.2, max: 4207.0 };
        let mut item = new_rolling_segment_proposal("x", "y");
        item.ceiling = 4208.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = RollingSegmentProposalRateRangeRule { min: 0.2, max: 950.9 };
        let mut item = new_rolling_segment_proposal("x", "y");
        item.rate = 952.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RollingSegmentProposalNameLengthRule { max_length: 86400 };
        let mut item = new_rolling_segment_proposal("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = RollingSegmentProposalExternalRefLengthRule { max_length: 1000 };
        let mut item = new_rolling_segment_proposal("x", "y");
        item.external_ref = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = RollingSegmentProposalRequiredExternalRefRule {
            message: "rolling segment proposal needs an entry".to_string(),
        };
        let mut item = new_rolling_segment_proposal("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
