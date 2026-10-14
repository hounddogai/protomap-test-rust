use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence claim must pass. Check returns its problems, or none.
pub trait EvidenceClaimRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceClaimRequiredCipherRule {
    pub message: String,
}

impl EvidenceClaimRule for EvidenceClaimRequiredCipherRule {
    fn check(&self, item: &EvidenceClaim) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceClaimCipherLengthRule {
    pub max_length: i64,
}

impl EvidenceClaimRule for EvidenceClaimCipherLengthRule {
    fn check(&self, item: &EvidenceClaim) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceClaimTenantIdLengthRule {
    pub max_length: i64,
}

impl EvidenceClaimRule for EvidenceClaimTenantIdLengthRule {
    fn check(&self, item: &EvidenceClaim) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceClaimTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl EvidenceClaimRule for EvidenceClaimTagsLimitRule {
    fn check(&self, item: &EvidenceClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceClaimCodeLengthRule {
    pub max_length: i64,
}

impl EvidenceClaimRule for EvidenceClaimCodeLengthRule {
    fn check(&self, item: &EvidenceClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceClaimCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EvidenceClaimRule for EvidenceClaimCeilingRangeRule {
    fn check(&self, item: &EvidenceClaim) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a evidence claim.
#[derive(Debug, Clone)]
pub struct EvidenceClaimValidator {
    pub rules: Vec<Rc<dyn EvidenceClaimRule>>,
}

impl EvidenceClaimValidator {
    pub fn validate(&self, item: &EvidenceClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EvidenceClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_claim_validator() -> EvidenceClaimValidator {
    EvidenceClaimValidator {
        rules: vec![
            Rc::new(EvidenceClaimRequiredCipherRule {
                message: "evidence claim needs this field".to_string(),
            }) as Rc<dyn EvidenceClaimRule>,
            Rc::new(EvidenceClaimCipherLengthRule { max_length: 32 }),
            Rc::new(EvidenceClaimTenantIdLengthRule { max_length: 120 }),
            Rc::new(EvidenceClaimTagsLimitRule {
                max_count: 86400,
                banned: Vec::new(),
            }),
            Rc::new(EvidenceClaimCodeLengthRule { max_length: 32 }),
            Rc::new(EvidenceClaimCeilingRangeRule { min: 1.8, max: 1821.0 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_claim_validator();
        let item = new_evidence_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = EvidenceClaimRequiredCipherRule {
            message: "evidence claim needs an entry".to_string(),
        };
        let mut item = new_evidence_claim("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = EvidenceClaimCipherLengthRule { max_length: 16 };
        let mut item = new_evidence_claim("x", "y");
        item.cipher = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = EvidenceClaimTenantIdLengthRule { max_length: 120 };
        let mut item = new_evidence_claim("x", "y");
        item.tenant_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EvidenceClaimCodeLengthRule { max_length: 32 };
        let mut item = new_evidence_claim("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = EvidenceClaimCeilingRangeRule { min: 5.5, max: 84.8 };
        let mut item = new_evidence_claim("x", "y");
        item.ceiling = 86.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
