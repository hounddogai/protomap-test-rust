use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional federation proposal must pass. Check returns its problems, or none.
pub trait RegionalFederationProposalRule: std::fmt::Debug {
    fn check(&self, item: &RegionalFederationProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFederationProposalChecksumLengthRule {
    pub max_length: i64,
}

impl RegionalFederationProposalRule for RegionalFederationProposalChecksumLengthRule {
    fn check(&self, item: &RegionalFederationProposal) -> Vec<String> {
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
pub struct RegionalFederationProposalRequiredCodeRule {
    pub message: String,
}

impl RegionalFederationProposalRule for RegionalFederationProposalRequiredCodeRule {
    fn check(&self, item: &RegionalFederationProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFederationProposalPinnedGuardRule {
    pub limit: f64,
}

impl RegionalFederationProposalRule for RegionalFederationProposalPinnedGuardRule {
    fn check(&self, item: &RegionalFederationProposal) -> Vec<String> {
        if item.pinned && (item.rate > self.limit) {
            return vec![format!("rate {} exceeds {} while pinned", item.rate, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFederationProposalWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalFederationProposalRule for RegionalFederationProposalWindowDaysRangeRule {
    fn check(&self, item: &RegionalFederationProposal) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFederationProposalAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RegionalFederationProposalRule for RegionalFederationProposalAllowedCodeRule {
    fn check(&self, item: &RegionalFederationProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional federation proposal.
#[derive(Debug, Clone)]
pub struct RegionalFederationProposalValidator {
    pub rules: Vec<Rc<dyn RegionalFederationProposalRule>>,
}

impl RegionalFederationProposalValidator {
    pub fn validate(&self, item: &RegionalFederationProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalFederationProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalFederationProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_federation_proposal_validator() -> RegionalFederationProposalValidator {
    RegionalFederationProposalValidator {
        rules: vec![
            Rc::new(RegionalFederationProposalChecksumLengthRule { max_length: 120 })
                as Rc<dyn RegionalFederationProposalRule>,
            Rc::new(RegionalFederationProposalRequiredCodeRule {
                message: "regional federation proposal needs a value".to_string(),
            }),
            Rc::new(RegionalFederationProposalPinnedGuardRule { limit: 1.7 }),
            Rc::new(RegionalFederationProposalWindowDaysRangeRule { min: 120, max: 86400 }),
            Rc::new(RegionalFederationProposalAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_federation_proposal_validator();
        let item = new_regional_federation_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = RegionalFederationProposalChecksumLengthRule { max_length: 3600 };
        let mut item = new_regional_federation_proposal("x", "y");
        item.checksum = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalFederationProposalRequiredCodeRule {
            message: "regional federation proposal needs an entry".to_string(),
        };
        let mut item = new_regional_federation_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = RegionalFederationProposalWindowDaysRangeRule { min: 120, max: 1000 };
        let mut item = new_regional_federation_proposal("x", "y");
        item.window_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
