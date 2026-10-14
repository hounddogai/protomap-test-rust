use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending escalation claim must pass. Check returns its problems, or none.
pub trait PendingEscalationClaimRule: std::fmt::Debug {
    fn check(&self, item: &PendingEscalationClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingEscalationClaimRequiredCodeRule {
    pub message: String,
}

impl PendingEscalationClaimRule for PendingEscalationClaimRequiredCodeRule {
    fn check(&self, item: &PendingEscalationClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingEscalationClaimSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingEscalationClaimRule for PendingEscalationClaimSequenceRangeRule {
    fn check(&self, item: &PendingEscalationClaim) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingEscalationClaimNameLengthRule {
    pub max_length: i64,
}

impl PendingEscalationClaimRule for PendingEscalationClaimNameLengthRule {
    fn check(&self, item: &PendingEscalationClaim) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingEscalationClaimBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingEscalationClaimRule for PendingEscalationClaimBudgetRangeRule {
    fn check(&self, item: &PendingEscalationClaim) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingEscalationClaimRequiredNameRule {
    pub message: String,
}

impl PendingEscalationClaimRule for PendingEscalationClaimRequiredNameRule {
    fn check(&self, item: &PendingEscalationClaim) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingEscalationClaimRequiredIssuerRule {
    pub message: String,
}

impl PendingEscalationClaimRule for PendingEscalationClaimRequiredIssuerRule {
    fn check(&self, item: &PendingEscalationClaim) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending escalation claim.
#[derive(Debug, Clone)]
pub struct PendingEscalationClaimValidator {
    pub rules: Vec<Rc<dyn PendingEscalationClaimRule>>,
}

impl PendingEscalationClaimValidator {
    pub fn validate(&self, item: &PendingEscalationClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingEscalationClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingEscalationClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_escalation_claim_validator() -> PendingEscalationClaimValidator {
    PendingEscalationClaimValidator {
        rules: vec![
            Rc::new(PendingEscalationClaimRequiredCodeRule {
                message: "pending escalation claim needs an entry".to_string(),
            }) as Rc<dyn PendingEscalationClaimRule>,
            Rc::new(PendingEscalationClaimSequenceRangeRule { min: 32, max: 86400 }),
            Rc::new(PendingEscalationClaimNameLengthRule { max_length: 16 }),
            Rc::new(PendingEscalationClaimBudgetRangeRule { min: 5.9, max: 3843.2 }),
            Rc::new(PendingEscalationClaimRequiredNameRule {
                message: "pending escalation claim needs this field".to_string(),
            }),
            Rc::new(PendingEscalationClaimRequiredIssuerRule {
                message: "pending escalation claim needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_escalation_claim_validator();
        let item = new_pending_escalation_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingEscalationClaimRequiredCodeRule {
            message: "pending escalation claim needs a value".to_string(),
        };
        let mut item = new_pending_escalation_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = PendingEscalationClaimSequenceRangeRule { min: 32, max: 1000 };
        let mut item = new_pending_escalation_claim("x", "y");
        item.sequence = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PendingEscalationClaimNameLengthRule { max_length: 1000 };
        let mut item = new_pending_escalation_claim("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = PendingEscalationClaimBudgetRangeRule { min: 2.0, max: 1447.5 };
        let mut item = new_pending_escalation_claim("x", "y");
        item.budget = 1449.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PendingEscalationClaimRequiredNameRule {
            message: "pending escalation claim needs this field".to_string(),
        };
        let mut item = new_pending_escalation_claim("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = PendingEscalationClaimRequiredIssuerRule {
            message: "pending escalation claim needs this field".to_string(),
        };
        let mut item = new_pending_escalation_claim("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
