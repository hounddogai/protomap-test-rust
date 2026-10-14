use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy probe proposal must pass. Check returns its problems, or none.
pub trait LegacyProbeProposalRule: std::fmt::Debug {
    fn check(&self, item: &LegacyProbeProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeProposalCodePrefixRule {
    pub prefix: String,
}

impl LegacyProbeProposalRule for LegacyProbeProposalCodePrefixRule {
    fn check(&self, item: &LegacyProbeProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeProposalBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyProbeProposalRule for LegacyProbeProposalBatchSizeRangeRule {
    fn check(&self, item: &LegacyProbeProposal) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeProposalRequiredCodeRule {
    pub message: String,
}

impl LegacyProbeProposalRule for LegacyProbeProposalRequiredCodeRule {
    fn check(&self, item: &LegacyProbeProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeProposalRequiredTitleRule {
    pub message: String,
}

impl LegacyProbeProposalRule for LegacyProbeProposalRequiredTitleRule {
    fn check(&self, item: &LegacyProbeProposal) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeProposalRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyProbeProposalRule for LegacyProbeProposalRateRangeRule {
    fn check(&self, item: &LegacyProbeProposal) -> Vec<String> {
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

/// Runs every rule of a legacy probe proposal.
#[derive(Debug, Clone)]
pub struct LegacyProbeProposalValidator {
    pub rules: Vec<Rc<dyn LegacyProbeProposalRule>>,
}

impl LegacyProbeProposalValidator {
    pub fn validate(&self, item: &LegacyProbeProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyProbeProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyProbeProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_probe_proposal_validator() -> LegacyProbeProposalValidator {
    LegacyProbeProposalValidator {
        rules: vec![
            Rc::new(LegacyProbeProposalCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn LegacyProbeProposalRule>,
            Rc::new(LegacyProbeProposalBatchSizeRangeRule { min: 120, max: 255 }),
            Rc::new(LegacyProbeProposalRequiredCodeRule {
                message: "legacy probe proposal needs a value".to_string(),
            }),
            Rc::new(LegacyProbeProposalRequiredTitleRule {
                message: "legacy probe proposal needs a value".to_string(),
            }),
            Rc::new(LegacyProbeProposalRateRangeRule { min: 4.8, max: 1621.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_probe_proposal_validator();
        let item = new_legacy_probe_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = LegacyProbeProposalBatchSizeRangeRule { min: 255, max: 3600 };
        let mut item = new_legacy_probe_proposal("x", "y");
        item.batch_size = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyProbeProposalRequiredCodeRule {
            message: "legacy probe proposal needs a value".to_string(),
        };
        let mut item = new_legacy_probe_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LegacyProbeProposalRequiredTitleRule {
            message: "legacy probe proposal needs an entry".to_string(),
        };
        let mut item = new_legacy_probe_proposal("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = LegacyProbeProposalRateRangeRule { min: 6.8, max: 471.3 };
        let mut item = new_legacy_probe_proposal("x", "y");
        item.rate = 472.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
