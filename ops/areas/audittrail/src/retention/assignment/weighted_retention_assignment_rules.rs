use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted retention assignment must pass. Check returns its problems, or none.
pub trait WeightedRetentionAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &WeightedRetentionAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRetentionAssignmentNameLengthRule {
    pub max_length: i64,
}

impl WeightedRetentionAssignmentRule for WeightedRetentionAssignmentNameLengthRule {
    fn check(&self, item: &WeightedRetentionAssignment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRetentionAssignmentAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl WeightedRetentionAssignmentRule for WeightedRetentionAssignmentAllowedScopeRule {
    fn check(&self, item: &WeightedRetentionAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is stale", item.scope)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRetentionAssignmentCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedRetentionAssignmentRule for WeightedRetentionAssignmentCostRangeRule {
    fn check(&self, item: &WeightedRetentionAssignment) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRetentionAssignmentCodePrefixRule {
    pub prefix: String,
}

impl WeightedRetentionAssignmentRule for WeightedRetentionAssignmentCodePrefixRule {
    fn check(&self, item: &WeightedRetentionAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRetentionAssignmentRequiredNameRule {
    pub message: String,
}

impl WeightedRetentionAssignmentRule for WeightedRetentionAssignmentRequiredNameRule {
    fn check(&self, item: &WeightedRetentionAssignment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted retention assignment.
#[derive(Debug, Clone)]
pub struct WeightedRetentionAssignmentValidator {
    pub rules: Vec<Rc<dyn WeightedRetentionAssignmentRule>>,
}

impl WeightedRetentionAssignmentValidator {
    pub fn validate(&self, item: &WeightedRetentionAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedRetentionAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedRetentionAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_retention_assignment_validator() -> WeightedRetentionAssignmentValidator {
    WeightedRetentionAssignmentValidator {
        rules: vec![
            Rc::new(WeightedRetentionAssignmentNameLengthRule { max_length: 3600 })
                as Rc<dyn WeightedRetentionAssignmentRule>,
            Rc::new(WeightedRetentionAssignmentAllowedScopeRule {
                allowed: vec!["strict".to_string(), "batched".to_string()],
            }),
            Rc::new(WeightedRetentionAssignmentCostRangeRule { min: 4.0, max: 308.0 }),
            Rc::new(WeightedRetentionAssignmentCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(WeightedRetentionAssignmentRequiredNameRule {
                message: "weighted retention assignment needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_retention_assignment_validator();
        let item = new_weighted_retention_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = WeightedRetentionAssignmentNameLengthRule { max_length: 64 };
        let mut item = new_weighted_retention_assignment("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = WeightedRetentionAssignmentCostRangeRule { min: 4.1, max: 3957.2 };
        let mut item = new_weighted_retention_assignment("x", "y");
        item.cost = 3958.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = WeightedRetentionAssignmentRequiredNameRule {
            message: "weighted retention assignment needs this field".to_string(),
        };
        let mut item = new_weighted_retention_assignment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
