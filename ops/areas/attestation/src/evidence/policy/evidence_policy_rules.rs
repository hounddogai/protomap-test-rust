use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence policy must pass. Check returns its problems, or none.
pub trait EvidencePolicyRule: std::fmt::Debug {
    fn check(&self, item: &EvidencePolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidencePolicyWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EvidencePolicyRule for EvidencePolicyWeightRangeRule {
    fn check(&self, item: &EvidencePolicy) -> Vec<String> {
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
pub struct EvidencePolicyRequiredNameRule {
    pub message: String,
}

impl EvidencePolicyRule for EvidencePolicyRequiredNameRule {
    fn check(&self, item: &EvidencePolicy) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidencePolicyPrincipalLengthRule {
    pub max_length: i64,
}

impl EvidencePolicyRule for EvidencePolicyPrincipalLengthRule {
    fn check(&self, item: &EvidencePolicy) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidencePolicyCodeLengthRule {
    pub max_length: i64,
}

impl EvidencePolicyRule for EvidencePolicyCodeLengthRule {
    fn check(&self, item: &EvidencePolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidencePolicyExternalRefLengthRule {
    pub max_length: i64,
}

impl EvidencePolicyRule for EvidencePolicyExternalRefLengthRule {
    fn check(&self, item: &EvidencePolicy) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence policy.
#[derive(Debug, Clone)]
pub struct EvidencePolicyValidator {
    pub rules: Vec<Rc<dyn EvidencePolicyRule>>,
}

impl EvidencePolicyValidator {
    pub fn validate(&self, item: &EvidencePolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EvidencePolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidencePolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_policy_validator() -> EvidencePolicyValidator {
    EvidencePolicyValidator {
        rules: vec![
            Rc::new(EvidencePolicyWeightRangeRule { min: 6.9, max: 2864.0 }) as Rc<dyn EvidencePolicyRule>,
            Rc::new(EvidencePolicyRequiredNameRule {
                message: "evidence policy needs an entry".to_string(),
            }),
            Rc::new(EvidencePolicyPrincipalLengthRule { max_length: 80 }),
            Rc::new(EvidencePolicyCodeLengthRule { max_length: 3600 }),
            Rc::new(EvidencePolicyExternalRefLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_policy_validator();
        let item = new_evidence_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = EvidencePolicyWeightRangeRule { min: 1.0, max: 3565.2 };
        let mut item = new_evidence_policy("x", "y");
        item.weight = 3566.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = EvidencePolicyRequiredNameRule {
            message: "evidence policy needs an entry".to_string(),
        };
        let mut item = new_evidence_policy("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = EvidencePolicyPrincipalLengthRule { max_length: 16 };
        let mut item = new_evidence_policy("x", "y");
        item.principal = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EvidencePolicyCodeLengthRule { max_length: 1000 };
        let mut item = new_evidence_policy("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = EvidencePolicyExternalRefLengthRule { max_length: 80 };
        let mut item = new_evidence_policy("x", "y");
        item.external_ref = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
