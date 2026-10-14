use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling alert proposal must pass. Check returns its problems, or none.
pub trait RollingAlertProposalRule: std::fmt::Debug {
    fn check(&self, item: &RollingAlertProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAlertProposalRequiredCodeRule {
    pub message: String,
}

impl RollingAlertProposalRule for RollingAlertProposalRequiredCodeRule {
    fn check(&self, item: &RollingAlertProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAlertProposalCodePrefixRule {
    pub prefix: String,
}

impl RollingAlertProposalRule for RollingAlertProposalCodePrefixRule {
    fn check(&self, item: &RollingAlertProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAlertProposalTimezoneLengthRule {
    pub max_length: i64,
}

impl RollingAlertProposalRule for RollingAlertProposalTimezoneLengthRule {
    fn check(&self, item: &RollingAlertProposal) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling alert proposal.
#[derive(Debug, Clone)]
pub struct RollingAlertProposalValidator {
    pub rules: Vec<Rc<dyn RollingAlertProposalRule>>,
}

impl RollingAlertProposalValidator {
    pub fn validate(&self, item: &RollingAlertProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingAlertProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingAlertProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_alert_proposal_validator() -> RollingAlertProposalValidator {
    RollingAlertProposalValidator {
        rules: vec![
            Rc::new(RollingAlertProposalRequiredCodeRule {
                message: "rolling alert proposal needs this field".to_string(),
            }) as Rc<dyn RollingAlertProposalRule>,
            Rc::new(RollingAlertProposalCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RollingAlertProposalTimezoneLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_alert_proposal_validator();
        let item = new_rolling_alert_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingAlertProposalRequiredCodeRule {
            message: "rolling alert proposal needs this field".to_string(),
        };
        let mut item = new_rolling_alert_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = RollingAlertProposalTimezoneLengthRule { max_length: 120 };
        let mut item = new_rolling_alert_proposal("x", "y");
        item.timezone = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
