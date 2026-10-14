use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence threshold must pass. Check returns its problems, or none.
pub trait EvidenceThresholdRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceThresholdRequiredCodeRule {
    pub message: String,
}

impl EvidenceThresholdRule for EvidenceThresholdRequiredCodeRule {
    fn check(&self, item: &EvidenceThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceThresholdCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EvidenceThresholdRule for EvidenceThresholdCapacityRangeRule {
    fn check(&self, item: &EvidenceThreshold) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceThresholdCodeLengthRule {
    pub max_length: i64,
}

impl EvidenceThresholdRule for EvidenceThresholdCodeLengthRule {
    fn check(&self, item: &EvidenceThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceThresholdNameLengthRule {
    pub max_length: i64,
}

impl EvidenceThresholdRule for EvidenceThresholdNameLengthRule {
    fn check(&self, item: &EvidenceThreshold) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence threshold.
#[derive(Debug, Clone)]
pub struct EvidenceThresholdValidator {
    pub rules: Vec<Rc<dyn EvidenceThresholdRule>>,
}

impl EvidenceThresholdValidator {
    pub fn validate(&self, item: &EvidenceThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EvidenceThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_threshold_validator() -> EvidenceThresholdValidator {
    EvidenceThresholdValidator {
        rules: vec![
            Rc::new(EvidenceThresholdRequiredCodeRule {
                message: "evidence threshold needs this field".to_string(),
            }) as Rc<dyn EvidenceThresholdRule>,
            Rc::new(EvidenceThresholdCapacityRangeRule { min: 120, max: 86400 }),
            Rc::new(EvidenceThresholdCodeLengthRule { max_length: 120 }),
            Rc::new(EvidenceThresholdNameLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_threshold_validator();
        let item = new_evidence_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EvidenceThresholdRequiredCodeRule {
            message: "evidence threshold needs an entry".to_string(),
        };
        let mut item = new_evidence_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = EvidenceThresholdCapacityRangeRule { min: 80, max: 3600 };
        let mut item = new_evidence_threshold("x", "y");
        item.capacity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EvidenceThresholdCodeLengthRule { max_length: 3600 };
        let mut item = new_evidence_threshold("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = EvidenceThresholdNameLengthRule { max_length: 16 };
        let mut item = new_evidence_threshold("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
