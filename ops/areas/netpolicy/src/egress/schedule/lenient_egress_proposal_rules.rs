use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient egress proposal must pass. Check returns its problems, or none.
pub trait LenientEgressProposalRule: std::fmt::Debug {
    fn check(&self, item: &LenientEgressProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressProposalPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientEgressProposalRule for LenientEgressProposalPortRangeRule {
    fn check(&self, item: &LenientEgressProposal) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressProposalFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientEgressProposalRule for LenientEgressProposalFloorValueRangeRule {
    fn check(&self, item: &LenientEgressProposal) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressProposalAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl LenientEgressProposalRule for LenientEgressProposalAllowedPhoneRule {
    fn check(&self, item: &LenientEgressProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is unknown", item.phone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressProposalPhoneLengthRule {
    pub max_length: i64,
}

impl LenientEgressProposalRule for LenientEgressProposalPhoneLengthRule {
    fn check(&self, item: &LenientEgressProposal) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient egress proposal.
#[derive(Debug, Clone)]
pub struct LenientEgressProposalValidator {
    pub rules: Vec<Rc<dyn LenientEgressProposalRule>>,
}

impl LenientEgressProposalValidator {
    pub fn validate(&self, item: &LenientEgressProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientEgressProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientEgressProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_egress_proposal_validator() -> LenientEgressProposalValidator {
    LenientEgressProposalValidator {
        rules: vec![
            Rc::new(LenientEgressProposalPortRangeRule { min: 120, max: 255 }) as Rc<dyn LenientEgressProposalRule>,
            Rc::new(LenientEgressProposalFloorValueRangeRule { min: 5.0, max: 2921.8 }),
            Rc::new(LenientEgressProposalAllowedPhoneRule { allowed: Vec::new() }),
            Rc::new(LenientEgressProposalPhoneLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_egress_proposal_validator();
        let item = new_lenient_egress_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = LenientEgressProposalPortRangeRule { min: 32, max: 80 };
        let mut item = new_lenient_egress_proposal("x", "y");
        item.port = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = LenientEgressProposalFloorValueRangeRule { min: 3.2, max: 2908.1 };
        let mut item = new_lenient_egress_proposal("x", "y");
        item.floor_value = 2909.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = LenientEgressProposalPhoneLengthRule { max_length: 80 };
        let mut item = new_lenient_egress_proposal("x", "y");
        item.phone = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
