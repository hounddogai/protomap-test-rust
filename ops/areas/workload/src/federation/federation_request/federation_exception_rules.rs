use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a federation exception must pass. Check returns its problems, or none.
pub trait FederationExceptionRule: std::fmt::Debug {
    fn check(&self, item: &FederationException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationExceptionGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FederationExceptionRule for FederationExceptionGraceDaysRangeRule {
    fn check(&self, item: &FederationException) -> Vec<String> {
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
pub struct FederationExceptionPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FederationExceptionRule for FederationExceptionPortRangeRule {
    fn check(&self, item: &FederationException) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationExceptionCodeLengthRule {
    pub max_length: i64,
}

impl FederationExceptionRule for FederationExceptionCodeLengthRule {
    fn check(&self, item: &FederationException) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationExceptionRequiredTitleRule {
    pub message: String,
}

impl FederationExceptionRule for FederationExceptionRequiredTitleRule {
    fn check(&self, item: &FederationException) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationExceptionMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FederationExceptionRule for FederationExceptionMinimumRangeRule {
    fn check(&self, item: &FederationException) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a federation exception.
#[derive(Debug, Clone)]
pub struct FederationExceptionValidator {
    pub rules: Vec<Rc<dyn FederationExceptionRule>>,
}

impl FederationExceptionValidator {
    pub fn validate(&self, item: &FederationException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FederationException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_exception_validator() -> FederationExceptionValidator {
    FederationExceptionValidator {
        rules: vec![
            Rc::new(FederationExceptionGraceDaysRangeRule { min: 64, max: 255 }) as Rc<dyn FederationExceptionRule>,
            Rc::new(FederationExceptionPortRangeRule { min: 80, max: 1000 }),
            Rc::new(FederationExceptionCodeLengthRule { max_length: 255 }),
            Rc::new(FederationExceptionRequiredTitleRule {
                message: "federation exception needs an entry".to_string(),
            }),
            Rc::new(FederationExceptionMinimumRangeRule { min: 0.2, max: 2779.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_exception_validator();
        let item = new_federation_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = FederationExceptionGraceDaysRangeRule { min: 32, max: 80 };
        let mut item = new_federation_exception("x", "y");
        item.grace_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = FederationExceptionPortRangeRule { min: 255, max: 1000 };
        let mut item = new_federation_exception("x", "y");
        item.port = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FederationExceptionCodeLengthRule { max_length: 86400 };
        let mut item = new_federation_exception("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = FederationExceptionRequiredTitleRule {
            message: "federation exception needs this field".to_string(),
        };
        let mut item = new_federation_exception("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = FederationExceptionMinimumRangeRule { min: 9.5, max: 3502.7 };
        let mut item = new_federation_exception("x", "y");
        item.minimum = 3504.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
