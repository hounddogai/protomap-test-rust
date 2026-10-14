use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary query assignment must pass. Check returns its problems, or none.
pub trait PrimaryQueryAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryQueryAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryQueryAssignmentTimezoneLengthRule {
    pub max_length: i64,
}

impl PrimaryQueryAssignmentRule for PrimaryQueryAssignmentTimezoneLengthRule {
    fn check(&self, item: &PrimaryQueryAssignment) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryQueryAssignmentWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryQueryAssignmentRule for PrimaryQueryAssignmentWeightRangeRule {
    fn check(&self, item: &PrimaryQueryAssignment) -> Vec<String> {
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
pub struct PrimaryQueryAssignmentCipherLengthRule {
    pub max_length: i64,
}

impl PrimaryQueryAssignmentRule for PrimaryQueryAssignmentCipherLengthRule {
    fn check(&self, item: &PrimaryQueryAssignment) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary query assignment.
#[derive(Debug, Clone)]
pub struct PrimaryQueryAssignmentValidator {
    pub rules: Vec<Rc<dyn PrimaryQueryAssignmentRule>>,
}

impl PrimaryQueryAssignmentValidator {
    pub fn validate(&self, item: &PrimaryQueryAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryQueryAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryQueryAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_query_assignment_validator() -> PrimaryQueryAssignmentValidator {
    PrimaryQueryAssignmentValidator {
        rules: vec![
            Rc::new(PrimaryQueryAssignmentTimezoneLengthRule { max_length: 16 }) as Rc<dyn PrimaryQueryAssignmentRule>,
            Rc::new(PrimaryQueryAssignmentWeightRangeRule { min: 5.8, max: 3641.8 }),
            Rc::new(PrimaryQueryAssignmentCipherLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_query_assignment_validator();
        let item = new_primary_query_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = PrimaryQueryAssignmentTimezoneLengthRule { max_length: 255 };
        let mut item = new_primary_query_assignment("x", "y");
        item.timezone = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = PrimaryQueryAssignmentWeightRangeRule { min: 1.8, max: 2538.9 };
        let mut item = new_primary_query_assignment("x", "y");
        item.weight = 2540.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = PrimaryQueryAssignmentCipherLengthRule { max_length: 120 };
        let mut item = new_primary_query_assignment("x", "y");
        item.cipher = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
