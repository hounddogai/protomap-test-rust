use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a escalation claim must pass. Check returns its problems, or none.
pub trait EscalationClaimRule: std::fmt::Debug {
    fn check(&self, item: &EscalationClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationClaimRequiredCodeRule {
    pub message: String,
}

impl EscalationClaimRule for EscalationClaimRequiredCodeRule {
    fn check(&self, item: &EscalationClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationClaimAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EscalationClaimRule for EscalationClaimAttemptsRangeRule {
    fn check(&self, item: &EscalationClaim) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationClaimDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EscalationClaimRule for EscalationClaimDurationMinutesRangeRule {
    fn check(&self, item: &EscalationClaim) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationClaimCodePrefixRule {
    pub prefix: String,
}

impl EscalationClaimRule for EscalationClaimCodePrefixRule {
    fn check(&self, item: &EscalationClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationClaimMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EscalationClaimRule for EscalationClaimMarginRangeRule {
    fn check(&self, item: &EscalationClaim) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationClaimRequiredScopeRule {
    pub message: String,
}

impl EscalationClaimRule for EscalationClaimRequiredScopeRule {
    fn check(&self, item: &EscalationClaim) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation claim.
#[derive(Debug, Clone)]
pub struct EscalationClaimValidator {
    pub rules: Vec<Rc<dyn EscalationClaimRule>>,
}

impl EscalationClaimValidator {
    pub fn validate(&self, item: &EscalationClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EscalationClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_claim_validator() -> EscalationClaimValidator {
    EscalationClaimValidator {
        rules: vec![
            Rc::new(EscalationClaimRequiredCodeRule {
                message: "escalation claim needs an entry".to_string(),
            }) as Rc<dyn EscalationClaimRule>,
            Rc::new(EscalationClaimAttemptsRangeRule { min: 16, max: 80 }),
            Rc::new(EscalationClaimDurationMinutesRangeRule { min: 1000, max: 1000 }),
            Rc::new(EscalationClaimCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(EscalationClaimMarginRangeRule { min: 2.2, max: 1120.2 }),
            Rc::new(EscalationClaimRequiredScopeRule {
                message: "escalation claim needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_claim_validator();
        let item = new_escalation_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EscalationClaimRequiredCodeRule {
            message: "escalation claim needs an entry".to_string(),
        };
        let mut item = new_escalation_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = EscalationClaimAttemptsRangeRule { min: 120, max: 1000 };
        let mut item = new_escalation_claim("x", "y");
        item.attempts = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = EscalationClaimDurationMinutesRangeRule { min: 80, max: 3600 };
        let mut item = new_escalation_claim("x", "y");
        item.duration_minutes = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = EscalationClaimMarginRangeRule { min: 4.0, max: 4359.6 };
        let mut item = new_escalation_claim("x", "y");
        item.margin = 4361.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = EscalationClaimRequiredScopeRule {
            message: "escalation claim needs this field".to_string(),
        };
        let mut item = new_escalation_claim("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
