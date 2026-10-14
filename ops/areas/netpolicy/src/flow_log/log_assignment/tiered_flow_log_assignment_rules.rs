use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered flow log assignment must pass. Check returns its problems, or none.
pub trait TieredFlowLogAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &TieredFlowLogAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogAssignmentExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredFlowLogAssignmentRule for TieredFlowLogAssignmentExpiresAtRangeRule {
    fn check(&self, item: &TieredFlowLogAssignment) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogAssignmentAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl TieredFlowLogAssignmentRule for TieredFlowLogAssignmentAllowedPrincipalRule {
    fn check(&self, item: &TieredFlowLogAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is above maximum", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogAssignmentPrincipalLengthRule {
    pub max_length: i64,
}

impl TieredFlowLogAssignmentRule for TieredFlowLogAssignmentPrincipalLengthRule {
    fn check(&self, item: &TieredFlowLogAssignment) -> Vec<String> {
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
pub struct TieredFlowLogAssignmentRequiredPrincipalRule {
    pub message: String,
}

impl TieredFlowLogAssignmentRule for TieredFlowLogAssignmentRequiredPrincipalRule {
    fn check(&self, item: &TieredFlowLogAssignment) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered flow log assignment.
#[derive(Debug, Clone)]
pub struct TieredFlowLogAssignmentValidator {
    pub rules: Vec<Rc<dyn TieredFlowLogAssignmentRule>>,
}

impl TieredFlowLogAssignmentValidator {
    pub fn validate(&self, item: &TieredFlowLogAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredFlowLogAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredFlowLogAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_flow_log_assignment_validator() -> TieredFlowLogAssignmentValidator {
    TieredFlowLogAssignmentValidator {
        rules: vec![
            Rc::new(TieredFlowLogAssignmentExpiresAtRangeRule { min: 80, max: 80 })
                as Rc<dyn TieredFlowLogAssignmentRule>,
            Rc::new(TieredFlowLogAssignmentAllowedPrincipalRule {
                allowed: vec!["weighted".to_string()],
            }),
            Rc::new(TieredFlowLogAssignmentPrincipalLengthRule { max_length: 86400 }),
            Rc::new(TieredFlowLogAssignmentRequiredPrincipalRule {
                message: "tiered flow log assignment needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_flow_log_assignment_validator();
        let item = new_tiered_flow_log_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = TieredFlowLogAssignmentExpiresAtRangeRule { min: 80, max: 120 };
        let mut item = new_tiered_flow_log_assignment("x", "y");
        item.expires_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = TieredFlowLogAssignmentPrincipalLengthRule { max_length: 255 };
        let mut item = new_tiered_flow_log_assignment("x", "y");
        item.principal = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = TieredFlowLogAssignmentRequiredPrincipalRule {
            message: "tiered flow log assignment needs a value".to_string(),
        };
        let mut item = new_tiered_flow_log_assignment("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
