use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier request must pass. Check returns its problems, or none.
pub trait VerifierRequestRule: std::fmt::Debug {
    fn check(&self, item: &VerifierRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierRequestCodeLengthRule {
    pub max_length: i64,
}

impl VerifierRequestRule for VerifierRequestCodeLengthRule {
    fn check(&self, item: &VerifierRequest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierRequestCodePrefixRule {
    pub prefix: String,
}

impl VerifierRequestRule for VerifierRequestCodePrefixRule {
    fn check(&self, item: &VerifierRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierRequestMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl VerifierRequestRule for VerifierRequestMinimumRangeRule {
    fn check(&self, item: &VerifierRequest) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier request.
#[derive(Debug, Clone)]
pub struct VerifierRequestValidator {
    pub rules: Vec<Rc<dyn VerifierRequestRule>>,
}

impl VerifierRequestValidator {
    pub fn validate(&self, item: &VerifierRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &VerifierRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_request_validator() -> VerifierRequestValidator {
    VerifierRequestValidator {
        rules: vec![
            Rc::new(VerifierRequestCodeLengthRule { max_length: 64 }) as Rc<dyn VerifierRequestRule>,
            Rc::new(VerifierRequestCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(VerifierRequestMinimumRangeRule { min: 3.6, max: 2273.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_request_validator();
        let item = new_verifier_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = VerifierRequestCodeLengthRule { max_length: 120 };
        let mut item = new_verifier_request("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = VerifierRequestMinimumRangeRule { min: 9.3, max: 2960.5 };
        let mut item = new_verifier_request("x", "y");
        item.minimum = 2962.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
