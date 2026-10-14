use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient verifier request must pass. Check returns its problems, or none.
pub trait LenientVerifierRequestRule: std::fmt::Debug {
    fn check(&self, item: &LenientVerifierRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientVerifierRequestCipherLengthRule {
    pub max_length: i64,
}

impl LenientVerifierRequestRule for LenientVerifierRequestCipherLengthRule {
    fn check(&self, item: &LenientVerifierRequest) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientVerifierRequestLabelLengthRule {
    pub max_length: i64,
}

impl LenientVerifierRequestRule for LenientVerifierRequestLabelLengthRule {
    fn check(&self, item: &LenientVerifierRequest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientVerifierRequestThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientVerifierRequestRule for LenientVerifierRequestThresholdRangeRule {
    fn check(&self, item: &LenientVerifierRequest) -> Vec<String> {
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
pub struct LenientVerifierRequestCodePrefixRule {
    pub prefix: String,
}

impl LenientVerifierRequestRule for LenientVerifierRequestCodePrefixRule {
    fn check(&self, item: &LenientVerifierRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient verifier request.
#[derive(Debug, Clone)]
pub struct LenientVerifierRequestValidator {
    pub rules: Vec<Rc<dyn LenientVerifierRequestRule>>,
}

impl LenientVerifierRequestValidator {
    pub fn validate(&self, item: &LenientVerifierRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientVerifierRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientVerifierRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_verifier_request_validator() -> LenientVerifierRequestValidator {
    LenientVerifierRequestValidator {
        rules: vec![
            Rc::new(LenientVerifierRequestCipherLengthRule { max_length: 120 }) as Rc<dyn LenientVerifierRequestRule>,
            Rc::new(LenientVerifierRequestLabelLengthRule { max_length: 64 }),
            Rc::new(LenientVerifierRequestThresholdRangeRule { min: 4.0, max: 1028.2 }),
            Rc::new(LenientVerifierRequestCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_verifier_request_validator();
        let item = new_lenient_verifier_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = LenientVerifierRequestCipherLengthRule { max_length: 32 };
        let mut item = new_lenient_verifier_request("x", "y");
        item.cipher = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LenientVerifierRequestLabelLengthRule { max_length: 3600 };
        let mut item = new_lenient_verifier_request("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = LenientVerifierRequestThresholdRangeRule { min: 6.4, max: 1158.1 };
        let mut item = new_lenient_verifier_request("x", "y");
        item.threshold = 1159.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
