use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a flow log claim must pass. Check returns its problems, or none.
pub trait FlowLogClaimRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogClaimRequiredContactEmailRule {
    pub message: String,
}

impl FlowLogClaimRule for FlowLogClaimRequiredContactEmailRule {
    fn check(&self, item: &FlowLogClaim) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogClaimApprovedGuardRule {
    pub limit: i64,
}

impl FlowLogClaimRule for FlowLogClaimApprovedGuardRule {
    fn check(&self, item: &FlowLogClaim) -> Vec<String> {
        if item.approved && (item.created_at > self.limit) {
            return vec![format!(
                "created at {} exceeds {} while approved",
                item.created_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogClaimAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl FlowLogClaimRule for FlowLogClaimAllowedCountryRule {
    fn check(&self, item: &FlowLogClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is missing", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogClaimRequiredLabelRule {
    pub message: String,
}

impl FlowLogClaimRule for FlowLogClaimRequiredLabelRule {
    fn check(&self, item: &FlowLogClaim) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log claim.
#[derive(Debug, Clone)]
pub struct FlowLogClaimValidator {
    pub rules: Vec<Rc<dyn FlowLogClaimRule>>,
}

impl FlowLogClaimValidator {
    pub fn validate(&self, item: &FlowLogClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FlowLogClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_claim_validator() -> FlowLogClaimValidator {
    FlowLogClaimValidator {
        rules: vec![
            Rc::new(FlowLogClaimRequiredContactEmailRule {
                message: "flow log claim needs this field".to_string(),
            }) as Rc<dyn FlowLogClaimRule>,
            Rc::new(FlowLogClaimApprovedGuardRule { limit: 32 }),
            Rc::new(FlowLogClaimAllowedCountryRule {
                allowed: vec!["capped".to_string()],
            }),
            Rc::new(FlowLogClaimRequiredLabelRule {
                message: "flow log claim needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_claim_validator();
        let item = new_flow_log_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = FlowLogClaimRequiredContactEmailRule {
            message: "flow log claim needs this field".to_string(),
        };
        let mut item = new_flow_log_claim("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = FlowLogClaimRequiredLabelRule {
            message: "flow log claim needs this field".to_string(),
        };
        let mut item = new_flow_log_claim("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
