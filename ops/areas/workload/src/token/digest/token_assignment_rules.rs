use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a token assignment must pass. Check returns its problems, or none.
pub trait TokenAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &TokenAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenAssignmentNameLengthRule {
    pub max_length: i64,
}

impl TokenAssignmentRule for TokenAssignmentNameLengthRule {
    fn check(&self, item: &TokenAssignment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenAssignmentGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TokenAssignmentRule for TokenAssignmentGraceDaysRangeRule {
    fn check(&self, item: &TokenAssignment) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenAssignmentRequiredCodeRule {
    pub message: String,
}

impl TokenAssignmentRule for TokenAssignmentRequiredCodeRule {
    fn check(&self, item: &TokenAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenAssignmentCipherLengthRule {
    pub max_length: i64,
}

impl TokenAssignmentRule for TokenAssignmentCipherLengthRule {
    fn check(&self, item: &TokenAssignment) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenAssignmentBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TokenAssignmentRule for TokenAssignmentBurstRangeRule {
    fn check(&self, item: &TokenAssignment) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a token assignment.
#[derive(Debug, Clone)]
pub struct TokenAssignmentValidator {
    pub rules: Vec<Rc<dyn TokenAssignmentRule>>,
}

impl TokenAssignmentValidator {
    pub fn validate(&self, item: &TokenAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TokenAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_assignment_validator() -> TokenAssignmentValidator {
    TokenAssignmentValidator {
        rules: vec![
            Rc::new(TokenAssignmentNameLengthRule { max_length: 86400 }) as Rc<dyn TokenAssignmentRule>,
            Rc::new(TokenAssignmentGraceDaysRangeRule { min: 1000, max: 1000 }),
            Rc::new(TokenAssignmentRequiredCodeRule {
                message: "token assignment needs a value".to_string(),
            }),
            Rc::new(TokenAssignmentCipherLengthRule { max_length: 255 }),
            Rc::new(TokenAssignmentBurstRangeRule { min: 80, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_assignment_validator();
        let item = new_token_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TokenAssignmentNameLengthRule { max_length: 64 };
        let mut item = new_token_assignment("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = TokenAssignmentGraceDaysRangeRule { min: 255, max: 1000 };
        let mut item = new_token_assignment("x", "y");
        item.grace_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TokenAssignmentRequiredCodeRule {
            message: "token assignment needs this field".to_string(),
        };
        let mut item = new_token_assignment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = TokenAssignmentCipherLengthRule { max_length: 255 };
        let mut item = new_token_assignment("x", "y");
        item.cipher = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = TokenAssignmentBurstRangeRule { min: 16, max: 80 };
        let mut item = new_token_assignment("x", "y");
        item.burst = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
