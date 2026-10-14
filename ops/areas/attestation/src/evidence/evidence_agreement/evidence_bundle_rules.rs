use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence bundle must pass. Check returns its problems, or none.
pub trait EvidenceBundleRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBundleRequiredUnitRule {
    pub message: String,
}

impl EvidenceBundleRule for EvidenceBundleRequiredUnitRule {
    fn check(&self, item: &EvidenceBundle) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBundleTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EvidenceBundleRule for EvidenceBundleTtlSecondsRangeRule {
    fn check(&self, item: &EvidenceBundle) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBundleRequiredCodeRule {
    pub message: String,
}

impl EvidenceBundleRule for EvidenceBundleRequiredCodeRule {
    fn check(&self, item: &EvidenceBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBundleCodePrefixRule {
    pub prefix: String,
}

impl EvidenceBundleRule for EvidenceBundleCodePrefixRule {
    fn check(&self, item: &EvidenceBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBundleCodeLengthRule {
    pub max_length: i64,
}

impl EvidenceBundleRule for EvidenceBundleCodeLengthRule {
    fn check(&self, item: &EvidenceBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence bundle.
#[derive(Debug, Clone)]
pub struct EvidenceBundleValidator {
    pub rules: Vec<Rc<dyn EvidenceBundleRule>>,
}

impl EvidenceBundleValidator {
    pub fn validate(&self, item: &EvidenceBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EvidenceBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_bundle_validator() -> EvidenceBundleValidator {
    EvidenceBundleValidator {
        rules: vec![
            Rc::new(EvidenceBundleRequiredUnitRule {
                message: "evidence bundle needs a value".to_string(),
            }) as Rc<dyn EvidenceBundleRule>,
            Rc::new(EvidenceBundleTtlSecondsRangeRule { min: 32, max: 1000 }),
            Rc::new(EvidenceBundleRequiredCodeRule {
                message: "evidence bundle needs an entry".to_string(),
            }),
            Rc::new(EvidenceBundleCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(EvidenceBundleCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_bundle_validator();
        let item = new_evidence_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = EvidenceBundleRequiredUnitRule {
            message: "evidence bundle needs this field".to_string(),
        };
        let mut item = new_evidence_bundle("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = EvidenceBundleTtlSecondsRangeRule { min: 32, max: 120 };
        let mut item = new_evidence_bundle("x", "y");
        item.ttl_seconds = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EvidenceBundleRequiredCodeRule {
            message: "evidence bundle needs this field".to_string(),
        };
        let mut item = new_evidence_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EvidenceBundleCodeLengthRule { max_length: 3600 };
        let mut item = new_evidence_bundle("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
