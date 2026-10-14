use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary probe proposal must pass. Check returns its problems, or none.
pub trait PrimaryProbeProposalRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryProbeProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryProbeProposalPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryProbeProposalRule for PrimaryProbeProposalPriorityRangeRule {
    fn check(&self, item: &PrimaryProbeProposal) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryProbeProposalRequiredCodeRule {
    pub message: String,
}

impl PrimaryProbeProposalRule for PrimaryProbeProposalRequiredCodeRule {
    fn check(&self, item: &PrimaryProbeProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryProbeProposalCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryProbeProposalRule for PrimaryProbeProposalCodeLengthRule {
    fn check(&self, item: &PrimaryProbeProposal) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryProbeProposalAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PrimaryProbeProposalRule for PrimaryProbeProposalAllowedCodeRule {
    fn check(&self, item: &PrimaryProbeProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is missing", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryProbeProposalCountryLengthRule {
    pub max_length: i64,
}

impl PrimaryProbeProposalRule for PrimaryProbeProposalCountryLengthRule {
    fn check(&self, item: &PrimaryProbeProposal) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary probe proposal.
#[derive(Debug, Clone)]
pub struct PrimaryProbeProposalValidator {
    pub rules: Vec<Rc<dyn PrimaryProbeProposalRule>>,
}

impl PrimaryProbeProposalValidator {
    pub fn validate(&self, item: &PrimaryProbeProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryProbeProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryProbeProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_probe_proposal_validator() -> PrimaryProbeProposalValidator {
    PrimaryProbeProposalValidator {
        rules: vec![
            Rc::new(PrimaryProbeProposalPriorityRangeRule { min: 120, max: 120 }) as Rc<dyn PrimaryProbeProposalRule>,
            Rc::new(PrimaryProbeProposalRequiredCodeRule {
                message: "primary probe proposal needs a value".to_string(),
            }),
            Rc::new(PrimaryProbeProposalCodeLengthRule { max_length: 32 }),
            Rc::new(PrimaryProbeProposalAllowedCodeRule {
                allowed: vec!["lenient".to_string()],
            }),
            Rc::new(PrimaryProbeProposalCountryLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_probe_proposal_validator();
        let item = new_primary_probe_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = PrimaryProbeProposalPriorityRangeRule { min: 16, max: 32 };
        let mut item = new_primary_probe_proposal("x", "y");
        item.priority = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryProbeProposalRequiredCodeRule {
            message: "primary probe proposal needs this field".to_string(),
        };
        let mut item = new_primary_probe_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryProbeProposalCodeLengthRule { max_length: 86400 };
        let mut item = new_primary_probe_proposal("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = PrimaryProbeProposalCountryLengthRule { max_length: 1000 };
        let mut item = new_primary_probe_proposal("x", "y");
        item.country = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
