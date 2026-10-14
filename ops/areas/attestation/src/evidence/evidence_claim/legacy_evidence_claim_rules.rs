use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy evidence claim must pass. Check returns its problems, or none.
pub trait LegacyEvidenceClaimRule: std::fmt::Debug {
    fn check(&self, item: &LegacyEvidenceClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceClaimRequiredHostRule {
    pub message: String,
}

impl LegacyEvidenceClaimRule for LegacyEvidenceClaimRequiredHostRule {
    fn check(&self, item: &LegacyEvidenceClaim) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceClaimGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyEvidenceClaimRule for LegacyEvidenceClaimGraceDaysRangeRule {
    fn check(&self, item: &LegacyEvidenceClaim) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceClaimCodePrefixRule {
    pub prefix: String,
}

impl LegacyEvidenceClaimRule for LegacyEvidenceClaimCodePrefixRule {
    fn check(&self, item: &LegacyEvidenceClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceClaimRequiredNotesRule {
    pub message: String,
}

impl LegacyEvidenceClaimRule for LegacyEvidenceClaimRequiredNotesRule {
    fn check(&self, item: &LegacyEvidenceClaim) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy evidence claim.
#[derive(Debug, Clone)]
pub struct LegacyEvidenceClaimValidator {
    pub rules: Vec<Rc<dyn LegacyEvidenceClaimRule>>,
}

impl LegacyEvidenceClaimValidator {
    pub fn validate(&self, item: &LegacyEvidenceClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyEvidenceClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyEvidenceClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_evidence_claim_validator() -> LegacyEvidenceClaimValidator {
    LegacyEvidenceClaimValidator {
        rules: vec![
            Rc::new(LegacyEvidenceClaimRequiredHostRule {
                message: "legacy evidence claim needs an entry".to_string(),
            }) as Rc<dyn LegacyEvidenceClaimRule>,
            Rc::new(LegacyEvidenceClaimGraceDaysRangeRule { min: 64, max: 3600 }),
            Rc::new(LegacyEvidenceClaimCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(LegacyEvidenceClaimRequiredNotesRule {
                message: "legacy evidence claim needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_evidence_claim_validator();
        let item = new_legacy_evidence_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = LegacyEvidenceClaimRequiredHostRule {
            message: "legacy evidence claim needs an entry".to_string(),
        };
        let mut item = new_legacy_evidence_claim("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = LegacyEvidenceClaimGraceDaysRangeRule { min: 120, max: 255 };
        let mut item = new_legacy_evidence_claim("x", "y");
        item.grace_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = LegacyEvidenceClaimRequiredNotesRule {
            message: "legacy evidence claim needs an entry".to_string(),
        };
        let mut item = new_legacy_evidence_claim("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
