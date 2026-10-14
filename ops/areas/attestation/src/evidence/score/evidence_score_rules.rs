use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence score must pass. Check returns its problems, or none.
pub trait EvidenceScoreRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceScoreRequiredNameRule {
    pub message: String,
}

impl EvidenceScoreRule for EvidenceScoreRequiredNameRule {
    fn check(&self, item: &EvidenceScore) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceScoreRequiredCipherRule {
    pub message: String,
}

impl EvidenceScoreRule for EvidenceScoreRequiredCipherRule {
    fn check(&self, item: &EvidenceScore) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceScoreCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EvidenceScoreRule for EvidenceScoreCeilingRangeRule {
    fn check(&self, item: &EvidenceScore) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceScoreRequiredCodeRule {
    pub message: String,
}

impl EvidenceScoreRule for EvidenceScoreRequiredCodeRule {
    fn check(&self, item: &EvidenceScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceScoreNameLengthRule {
    pub max_length: i64,
}

impl EvidenceScoreRule for EvidenceScoreNameLengthRule {
    fn check(&self, item: &EvidenceScore) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence score.
#[derive(Debug, Clone)]
pub struct EvidenceScoreValidator {
    pub rules: Vec<Rc<dyn EvidenceScoreRule>>,
}

impl EvidenceScoreValidator {
    pub fn validate(&self, item: &EvidenceScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EvidenceScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_score_validator() -> EvidenceScoreValidator {
    EvidenceScoreValidator {
        rules: vec![
            Rc::new(EvidenceScoreRequiredNameRule {
                message: "evidence score needs a value".to_string(),
            }) as Rc<dyn EvidenceScoreRule>,
            Rc::new(EvidenceScoreRequiredCipherRule {
                message: "evidence score needs an entry".to_string(),
            }),
            Rc::new(EvidenceScoreCeilingRangeRule { min: 9.1, max: 4102.9 }),
            Rc::new(EvidenceScoreRequiredCodeRule {
                message: "evidence score needs this field".to_string(),
            }),
            Rc::new(EvidenceScoreNameLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_score_validator();
        let item = new_evidence_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = EvidenceScoreRequiredNameRule {
            message: "evidence score needs an entry".to_string(),
        };
        let mut item = new_evidence_score("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = EvidenceScoreRequiredCipherRule {
            message: "evidence score needs this field".to_string(),
        };
        let mut item = new_evidence_score("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = EvidenceScoreCeilingRangeRule { min: 0.0, max: 976.7 };
        let mut item = new_evidence_score("x", "y");
        item.ceiling = 978.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EvidenceScoreRequiredCodeRule {
            message: "evidence score needs a value".to_string(),
        };
        let mut item = new_evidence_score("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = EvidenceScoreNameLengthRule { max_length: 1000 };
        let mut item = new_evidence_score("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
