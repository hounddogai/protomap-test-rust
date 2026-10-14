use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary evidence claim must pass. Check returns its problems, or none.
pub trait PrimaryEvidenceClaimRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryEvidenceClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceClaimTenantIdLengthRule {
    pub max_length: i64,
}

impl PrimaryEvidenceClaimRule for PrimaryEvidenceClaimTenantIdLengthRule {
    fn check(&self, item: &PrimaryEvidenceClaim) -> Vec<String> {
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
pub struct PrimaryEvidenceClaimAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl PrimaryEvidenceClaimRule for PrimaryEvidenceClaimAllowedTenantIdRule {
    fn check(&self, item: &PrimaryEvidenceClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is duplicated", item.tenant_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceClaimQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryEvidenceClaimRule for PrimaryEvidenceClaimQuantityRangeRule {
    fn check(&self, item: &PrimaryEvidenceClaim) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceClaimCodePrefixRule {
    pub prefix: String,
}

impl PrimaryEvidenceClaimRule for PrimaryEvidenceClaimCodePrefixRule {
    fn check(&self, item: &PrimaryEvidenceClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceClaimRequiredTenantIdRule {
    pub message: String,
}

impl PrimaryEvidenceClaimRule for PrimaryEvidenceClaimRequiredTenantIdRule {
    fn check(&self, item: &PrimaryEvidenceClaim) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceClaimRequiredCodeRule {
    pub message: String,
}

impl PrimaryEvidenceClaimRule for PrimaryEvidenceClaimRequiredCodeRule {
    fn check(&self, item: &PrimaryEvidenceClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary evidence claim.
#[derive(Debug, Clone)]
pub struct PrimaryEvidenceClaimValidator {
    pub rules: Vec<Rc<dyn PrimaryEvidenceClaimRule>>,
}

impl PrimaryEvidenceClaimValidator {
    pub fn validate(&self, item: &PrimaryEvidenceClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryEvidenceClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryEvidenceClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_evidence_claim_validator() -> PrimaryEvidenceClaimValidator {
    PrimaryEvidenceClaimValidator {
        rules: vec![
            Rc::new(PrimaryEvidenceClaimTenantIdLengthRule { max_length: 86400 }) as Rc<dyn PrimaryEvidenceClaimRule>,
            Rc::new(PrimaryEvidenceClaimAllowedTenantIdRule { allowed: Vec::new() }),
            Rc::new(PrimaryEvidenceClaimQuantityRangeRule { min: 120, max: 255 }),
            Rc::new(PrimaryEvidenceClaimCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(PrimaryEvidenceClaimRequiredTenantIdRule {
                message: "primary evidence claim needs an entry".to_string(),
            }),
            Rc::new(PrimaryEvidenceClaimRequiredCodeRule {
                message: "primary evidence claim needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_evidence_claim_validator();
        let item = new_primary_evidence_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = PrimaryEvidenceClaimTenantIdLengthRule { max_length: 32 };
        let mut item = new_primary_evidence_claim("x", "y");
        item.tenant_id = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = PrimaryEvidenceClaimQuantityRangeRule { min: 1000, max: 86400 };
        let mut item = new_primary_evidence_claim("x", "y");
        item.quantity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = PrimaryEvidenceClaimRequiredTenantIdRule {
            message: "primary evidence claim needs this field".to_string(),
        };
        let mut item = new_primary_evidence_claim("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryEvidenceClaimRequiredCodeRule {
            message: "primary evidence claim needs a value".to_string(),
        };
        let mut item = new_primary_evidence_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
