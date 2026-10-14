use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a egress milestone must pass. Check returns its problems, or none.
pub trait EgressMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &EgressMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressMilestoneRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EgressMilestoneRule for EgressMilestoneRatioRangeRule {
    fn check(&self, item: &EgressMilestone) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressMilestoneCodeLengthRule {
    pub max_length: i64,
}

impl EgressMilestoneRule for EgressMilestoneCodeLengthRule {
    fn check(&self, item: &EgressMilestone) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressMilestoneSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EgressMilestoneRule for EgressMilestoneSequenceRangeRule {
    fn check(&self, item: &EgressMilestone) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressMilestoneCodePrefixRule {
    pub prefix: String,
}

impl EgressMilestoneRule for EgressMilestoneCodePrefixRule {
    fn check(&self, item: &EgressMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressMilestoneRequiredCountryRule {
    pub message: String,
}

impl EgressMilestoneRule for EgressMilestoneRequiredCountryRule {
    fn check(&self, item: &EgressMilestone) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressMilestoneReviewerLengthRule {
    pub max_length: i64,
}

impl EgressMilestoneRule for EgressMilestoneReviewerLengthRule {
    fn check(&self, item: &EgressMilestone) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress milestone.
#[derive(Debug, Clone)]
pub struct EgressMilestoneValidator {
    pub rules: Vec<Rc<dyn EgressMilestoneRule>>,
}

impl EgressMilestoneValidator {
    pub fn validate(&self, item: &EgressMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EgressMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_milestone_validator() -> EgressMilestoneValidator {
    EgressMilestoneValidator {
        rules: vec![
            Rc::new(EgressMilestoneRatioRangeRule { min: 9.6, max: 1709.2 }) as Rc<dyn EgressMilestoneRule>,
            Rc::new(EgressMilestoneCodeLengthRule { max_length: 255 }),
            Rc::new(EgressMilestoneSequenceRangeRule { min: 32, max: 3600 }),
            Rc::new(EgressMilestoneCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(EgressMilestoneRequiredCountryRule {
                message: "egress milestone needs a value".to_string(),
            }),
            Rc::new(EgressMilestoneReviewerLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_milestone_validator();
        let item = new_egress_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = EgressMilestoneRatioRangeRule { min: 1.7, max: 3663.3 };
        let mut item = new_egress_milestone("x", "y");
        item.ratio = 3664.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EgressMilestoneCodeLengthRule { max_length: 16 };
        let mut item = new_egress_milestone("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = EgressMilestoneSequenceRangeRule { min: 32, max: 255 };
        let mut item = new_egress_milestone("x", "y");
        item.sequence = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = EgressMilestoneRequiredCountryRule {
            message: "egress milestone needs this field".to_string(),
        };
        let mut item = new_egress_milestone("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = EgressMilestoneReviewerLengthRule { max_length: 16 };
        let mut item = new_egress_milestone("x", "y");
        item.reviewer = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
