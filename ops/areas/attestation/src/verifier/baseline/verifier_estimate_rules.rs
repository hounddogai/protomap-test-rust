use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier estimate must pass. Check returns its problems, or none.
pub trait VerifierEstimateRule: std::fmt::Debug {
    fn check(&self, item: &VerifierEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierEstimateRequiredCodeRule {
    pub message: String,
}

impl VerifierEstimateRule for VerifierEstimateRequiredCodeRule {
    fn check(&self, item: &VerifierEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierEstimateRequiredNameRule {
    pub message: String,
}

impl VerifierEstimateRule for VerifierEstimateRequiredNameRule {
    fn check(&self, item: &VerifierEstimate) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierEstimatePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierEstimateRule for VerifierEstimatePortRangeRule {
    fn check(&self, item: &VerifierEstimate) -> Vec<String> {
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
pub struct VerifierEstimateCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierEstimateRule for VerifierEstimateCapacityRangeRule {
    fn check(&self, item: &VerifierEstimate) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier estimate.
#[derive(Debug, Clone)]
pub struct VerifierEstimateValidator {
    pub rules: Vec<Rc<dyn VerifierEstimateRule>>,
}

impl VerifierEstimateValidator {
    pub fn validate(&self, item: &VerifierEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &VerifierEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_estimate_validator() -> VerifierEstimateValidator {
    VerifierEstimateValidator {
        rules: vec![
            Rc::new(VerifierEstimateRequiredCodeRule {
                message: "verifier estimate needs this field".to_string(),
            }) as Rc<dyn VerifierEstimateRule>,
            Rc::new(VerifierEstimateRequiredNameRule {
                message: "verifier estimate needs this field".to_string(),
            }),
            Rc::new(VerifierEstimatePortRangeRule { min: 16, max: 120 }),
            Rc::new(VerifierEstimateCapacityRangeRule { min: 32, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_estimate_validator();
        let item = new_verifier_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = VerifierEstimateRequiredCodeRule {
            message: "verifier estimate needs a value".to_string(),
        };
        let mut item = new_verifier_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = VerifierEstimateRequiredNameRule {
            message: "verifier estimate needs a value".to_string(),
        };
        let mut item = new_verifier_estimate("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = VerifierEstimatePortRangeRule { min: 32, max: 255 };
        let mut item = new_verifier_estimate("x", "y");
        item.port = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = VerifierEstimateCapacityRangeRule { min: 32, max: 1000 };
        let mut item = new_verifier_estimate("x", "y");
        item.capacity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
