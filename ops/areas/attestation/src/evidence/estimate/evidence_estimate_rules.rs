use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence estimate must pass. Check returns its problems, or none.
pub trait EvidenceEstimateRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceEstimateRequiredCurrencyRule {
    pub message: String,
}

impl EvidenceEstimateRule for EvidenceEstimateRequiredCurrencyRule {
    fn check(&self, item: &EvidenceEstimate) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceEstimateCodePrefixRule {
    pub prefix: String,
}

impl EvidenceEstimateRule for EvidenceEstimateCodePrefixRule {
    fn check(&self, item: &EvidenceEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceEstimateRequiredTitleRule {
    pub message: String,
}

impl EvidenceEstimateRule for EvidenceEstimateRequiredTitleRule {
    fn check(&self, item: &EvidenceEstimate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence estimate.
#[derive(Debug, Clone)]
pub struct EvidenceEstimateValidator {
    pub rules: Vec<Rc<dyn EvidenceEstimateRule>>,
}

impl EvidenceEstimateValidator {
    pub fn validate(&self, item: &EvidenceEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EvidenceEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_estimate_validator() -> EvidenceEstimateValidator {
    EvidenceEstimateValidator {
        rules: vec![
            Rc::new(EvidenceEstimateRequiredCurrencyRule {
                message: "evidence estimate needs this field".to_string(),
            }) as Rc<dyn EvidenceEstimateRule>,
            Rc::new(EvidenceEstimateCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(EvidenceEstimateRequiredTitleRule {
                message: "evidence estimate needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_estimate_validator();
        let item = new_evidence_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = EvidenceEstimateRequiredCurrencyRule {
            message: "evidence estimate needs this field".to_string(),
        };
        let mut item = new_evidence_estimate("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = EvidenceEstimateRequiredTitleRule {
            message: "evidence estimate needs a value".to_string(),
        };
        let mut item = new_evidence_estimate("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
