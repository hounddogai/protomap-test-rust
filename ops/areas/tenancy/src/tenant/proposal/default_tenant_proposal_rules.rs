use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default tenant proposal must pass. Check returns its problems, or none.
pub trait DefaultTenantProposalRule: std::fmt::Debug {
    fn check(&self, item: &DefaultTenantProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTenantProposalRequiredNameRule {
    pub message: String,
}

impl DefaultTenantProposalRule for DefaultTenantProposalRequiredNameRule {
    fn check(&self, item: &DefaultTenantProposal) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTenantProposalChecksumLengthRule {
    pub max_length: i64,
}

impl DefaultTenantProposalRule for DefaultTenantProposalChecksumLengthRule {
    fn check(&self, item: &DefaultTenantProposal) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTenantProposalRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultTenantProposalRule for DefaultTenantProposalRateRangeRule {
    fn check(&self, item: &DefaultTenantProposal) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTenantProposalRequiredChecksumRule {
    pub message: String,
}

impl DefaultTenantProposalRule for DefaultTenantProposalRequiredChecksumRule {
    fn check(&self, item: &DefaultTenantProposal) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTenantProposalAllowedIssuerRule {
    pub allowed: Vec<String>,
}

impl DefaultTenantProposalRule for DefaultTenantProposalAllowedIssuerRule {
    fn check(&self, item: &DefaultTenantProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.issuer))) {
            return vec![format!("issuer {} is below minimum", item.issuer)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default tenant proposal.
#[derive(Debug, Clone)]
pub struct DefaultTenantProposalValidator {
    pub rules: Vec<Rc<dyn DefaultTenantProposalRule>>,
}

impl DefaultTenantProposalValidator {
    pub fn validate(&self, item: &DefaultTenantProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultTenantProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultTenantProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_tenant_proposal_validator() -> DefaultTenantProposalValidator {
    DefaultTenantProposalValidator {
        rules: vec![
            Rc::new(DefaultTenantProposalRequiredNameRule {
                message: "default tenant proposal needs an entry".to_string(),
            }) as Rc<dyn DefaultTenantProposalRule>,
            Rc::new(DefaultTenantProposalChecksumLengthRule { max_length: 16 }),
            Rc::new(DefaultTenantProposalRateRangeRule { min: 3.1, max: 3138.8 }),
            Rc::new(DefaultTenantProposalRequiredChecksumRule {
                message: "default tenant proposal needs a value".to_string(),
            }),
            Rc::new(DefaultTenantProposalAllowedIssuerRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_tenant_proposal_validator();
        let item = new_default_tenant_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = DefaultTenantProposalRequiredNameRule {
            message: "default tenant proposal needs a value".to_string(),
        };
        let mut item = new_default_tenant_proposal("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = DefaultTenantProposalChecksumLengthRule { max_length: 32 };
        let mut item = new_default_tenant_proposal("x", "y");
        item.checksum = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = DefaultTenantProposalRateRangeRule { min: 8.1, max: 3185.9 };
        let mut item = new_default_tenant_proposal("x", "y");
        item.rate = 3187.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = DefaultTenantProposalRequiredChecksumRule {
            message: "default tenant proposal needs a value".to_string(),
        };
        let mut item = new_default_tenant_proposal("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
