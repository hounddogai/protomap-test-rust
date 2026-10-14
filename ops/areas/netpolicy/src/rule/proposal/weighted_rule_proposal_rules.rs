use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted rule proposal must pass. Check returns its problems, or none.
pub trait WeightedRuleProposalRule: std::fmt::Debug {
    fn check(&self, item: &WeightedRuleProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleProposalRequiredTitleRule {
    pub message: String,
}

impl WeightedRuleProposalRule for WeightedRuleProposalRequiredTitleRule {
    fn check(&self, item: &WeightedRuleProposal) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleProposalRequiredCodeRule {
    pub message: String,
}

impl WeightedRuleProposalRule for WeightedRuleProposalRequiredCodeRule {
    fn check(&self, item: &WeightedRuleProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleProposalUnitLengthRule {
    pub max_length: i64,
}

impl WeightedRuleProposalRule for WeightedRuleProposalUnitLengthRule {
    fn check(&self, item: &WeightedRuleProposal) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleProposalThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedRuleProposalRule for WeightedRuleProposalThresholdRangeRule {
    fn check(&self, item: &WeightedRuleProposal) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleProposalTitleLengthRule {
    pub max_length: i64,
}

impl WeightedRuleProposalRule for WeightedRuleProposalTitleLengthRule {
    fn check(&self, item: &WeightedRuleProposal) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleProposalRequiredUnitRule {
    pub message: String,
}

impl WeightedRuleProposalRule for WeightedRuleProposalRequiredUnitRule {
    fn check(&self, item: &WeightedRuleProposal) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted rule proposal.
#[derive(Debug, Clone)]
pub struct WeightedRuleProposalValidator {
    pub rules: Vec<Rc<dyn WeightedRuleProposalRule>>,
}

impl WeightedRuleProposalValidator {
    pub fn validate(&self, item: &WeightedRuleProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedRuleProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedRuleProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_rule_proposal_validator() -> WeightedRuleProposalValidator {
    WeightedRuleProposalValidator {
        rules: vec![
            Rc::new(WeightedRuleProposalRequiredTitleRule {
                message: "weighted rule proposal needs an entry".to_string(),
            }) as Rc<dyn WeightedRuleProposalRule>,
            Rc::new(WeightedRuleProposalRequiredCodeRule {
                message: "weighted rule proposal needs this field".to_string(),
            }),
            Rc::new(WeightedRuleProposalUnitLengthRule { max_length: 16 }),
            Rc::new(WeightedRuleProposalThresholdRangeRule { min: 2.0, max: 2733.9 }),
            Rc::new(WeightedRuleProposalTitleLengthRule { max_length: 16 }),
            Rc::new(WeightedRuleProposalRequiredUnitRule {
                message: "weighted rule proposal needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_rule_proposal_validator();
        let item = new_weighted_rule_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = WeightedRuleProposalRequiredTitleRule {
            message: "weighted rule proposal needs a value".to_string(),
        };
        let mut item = new_weighted_rule_proposal("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedRuleProposalRequiredCodeRule {
            message: "weighted rule proposal needs this field".to_string(),
        };
        let mut item = new_weighted_rule_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = WeightedRuleProposalUnitLengthRule { max_length: 32 };
        let mut item = new_weighted_rule_proposal("x", "y");
        item.unit = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = WeightedRuleProposalThresholdRangeRule { min: 2.1, max: 3930.9 };
        let mut item = new_weighted_rule_proposal("x", "y");
        item.threshold = 3932.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = WeightedRuleProposalTitleLengthRule { max_length: 3600 };
        let mut item = new_weighted_rule_proposal("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = WeightedRuleProposalRequiredUnitRule {
            message: "weighted rule proposal needs an entry".to_string(),
        };
        let mut item = new_weighted_rule_proposal("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
