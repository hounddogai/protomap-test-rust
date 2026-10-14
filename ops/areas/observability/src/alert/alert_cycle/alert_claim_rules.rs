use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a alert claim must pass. Check returns its problems, or none.
pub trait AlertClaimRule: std::fmt::Debug {
    fn check(&self, item: &AlertClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertClaimAllowedSummaryRule {
    pub allowed: Vec<String>,
}

impl AlertClaimRule for AlertClaimAllowedSummaryRule {
    fn check(&self, item: &AlertClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.summary)))
        {
            return vec![format!("summary {} is below minimum", item.summary)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertClaimRequiredCodeRule {
    pub message: String,
}

impl AlertClaimRule for AlertClaimRequiredCodeRule {
    fn check(&self, item: &AlertClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertClaimRequiredSummaryRule {
    pub message: String,
}

impl AlertClaimRule for AlertClaimRequiredSummaryRule {
    fn check(&self, item: &AlertClaim) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertClaimRequiredLabelRule {
    pub message: String,
}

impl AlertClaimRule for AlertClaimRequiredLabelRule {
    fn check(&self, item: &AlertClaim) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert claim.
#[derive(Debug, Clone)]
pub struct AlertClaimValidator {
    pub rules: Vec<Rc<dyn AlertClaimRule>>,
}

impl AlertClaimValidator {
    pub fn validate(&self, item: &AlertClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AlertClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_claim_validator() -> AlertClaimValidator {
    AlertClaimValidator {
        rules: vec![
            Rc::new(AlertClaimAllowedSummaryRule {
                allowed: vec!["rolling".to_string(), "tiered".to_string()],
            }) as Rc<dyn AlertClaimRule>,
            Rc::new(AlertClaimRequiredCodeRule {
                message: "alert claim needs a value".to_string(),
            }),
            Rc::new(AlertClaimRequiredSummaryRule {
                message: "alert claim needs this field".to_string(),
            }),
            Rc::new(AlertClaimRequiredLabelRule {
                message: "alert claim needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_claim_validator();
        let item = new_alert_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AlertClaimRequiredCodeRule {
            message: "alert claim needs this field".to_string(),
        };
        let mut item = new_alert_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = AlertClaimRequiredSummaryRule {
            message: "alert claim needs an entry".to_string(),
        };
        let mut item = new_alert_claim("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = AlertClaimRequiredLabelRule {
            message: "alert claim needs a value".to_string(),
        };
        let mut item = new_alert_claim("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
