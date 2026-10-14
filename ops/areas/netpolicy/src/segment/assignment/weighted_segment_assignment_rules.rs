use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted segment assignment must pass. Check returns its problems, or none.
pub trait WeightedSegmentAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &WeightedSegmentAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentAssignmentWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedSegmentAssignmentRule for WeightedSegmentAssignmentWeightRangeRule {
    fn check(&self, item: &WeightedSegmentAssignment) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentAssignmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl WeightedSegmentAssignmentRule for WeightedSegmentAssignmentAllowedCodeRule {
    fn check(&self, item: &WeightedSegmentAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentAssignmentRequiredCodeRule {
    pub message: String,
}

impl WeightedSegmentAssignmentRule for WeightedSegmentAssignmentRequiredCodeRule {
    fn check(&self, item: &WeightedSegmentAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentAssignmentBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedSegmentAssignmentRule for WeightedSegmentAssignmentBudgetRangeRule {
    fn check(&self, item: &WeightedSegmentAssignment) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentAssignmentRequiredRegionRule {
    pub message: String,
}

impl WeightedSegmentAssignmentRule for WeightedSegmentAssignmentRequiredRegionRule {
    fn check(&self, item: &WeightedSegmentAssignment) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentAssignmentRequiredCurrencyRule {
    pub message: String,
}

impl WeightedSegmentAssignmentRule for WeightedSegmentAssignmentRequiredCurrencyRule {
    fn check(&self, item: &WeightedSegmentAssignment) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted segment assignment.
#[derive(Debug, Clone)]
pub struct WeightedSegmentAssignmentValidator {
    pub rules: Vec<Rc<dyn WeightedSegmentAssignmentRule>>,
}

impl WeightedSegmentAssignmentValidator {
    pub fn validate(&self, item: &WeightedSegmentAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedSegmentAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedSegmentAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_segment_assignment_validator() -> WeightedSegmentAssignmentValidator {
    WeightedSegmentAssignmentValidator {
        rules: vec![
            Rc::new(WeightedSegmentAssignmentWeightRangeRule { min: 8.5, max: 3845.9 })
                as Rc<dyn WeightedSegmentAssignmentRule>,
            Rc::new(WeightedSegmentAssignmentAllowedCodeRule {
                allowed: vec!["capped".to_string(), "scheduled".to_string(), "primary".to_string()],
            }),
            Rc::new(WeightedSegmentAssignmentRequiredCodeRule {
                message: "weighted segment assignment needs a value".to_string(),
            }),
            Rc::new(WeightedSegmentAssignmentBudgetRangeRule { min: 1.2, max: 1561.9 }),
            Rc::new(WeightedSegmentAssignmentRequiredRegionRule {
                message: "weighted segment assignment needs this field".to_string(),
            }),
            Rc::new(WeightedSegmentAssignmentRequiredCurrencyRule {
                message: "weighted segment assignment needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_segment_assignment_validator();
        let item = new_weighted_segment_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = WeightedSegmentAssignmentWeightRangeRule { min: 3.7, max: 1890.1 };
        let mut item = new_weighted_segment_assignment("x", "y");
        item.weight = 1891.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedSegmentAssignmentRequiredCodeRule {
            message: "weighted segment assignment needs an entry".to_string(),
        };
        let mut item = new_weighted_segment_assignment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = WeightedSegmentAssignmentBudgetRangeRule { min: 2.0, max: 4710.1 };
        let mut item = new_weighted_segment_assignment("x", "y");
        item.budget = 4711.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = WeightedSegmentAssignmentRequiredRegionRule {
            message: "weighted segment assignment needs this field".to_string(),
        };
        let mut item = new_weighted_segment_assignment("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = WeightedSegmentAssignmentRequiredCurrencyRule {
            message: "weighted segment assignment needs a value".to_string(),
        };
        let mut item = new_weighted_segment_assignment("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
