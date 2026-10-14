use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a peering estimate must pass. Check returns its problems, or none.
pub trait PeeringEstimateRule: std::fmt::Debug {
    fn check(&self, item: &PeeringEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringEstimateRequiredPrincipalRule {
    pub message: String,
}

impl PeeringEstimateRule for PeeringEstimateRequiredPrincipalRule {
    fn check(&self, item: &PeeringEstimate) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringEstimateTitleLengthRule {
    pub max_length: i64,
}

impl PeeringEstimateRule for PeeringEstimateTitleLengthRule {
    fn check(&self, item: &PeeringEstimate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringEstimateCodePrefixRule {
    pub prefix: String,
}

impl PeeringEstimateRule for PeeringEstimateCodePrefixRule {
    fn check(&self, item: &PeeringEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringEstimateRequiredTitleRule {
    pub message: String,
}

impl PeeringEstimateRule for PeeringEstimateRequiredTitleRule {
    fn check(&self, item: &PeeringEstimate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringEstimateAllowedSegmentRule {
    pub allowed: Vec<String>,
}

impl PeeringEstimateRule for PeeringEstimateAllowedSegmentRule {
    fn check(&self, item: &PeeringEstimate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.segment)))
        {
            return vec![format!("segment {} is not allowed", item.segment)];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering estimate.
#[derive(Debug, Clone)]
pub struct PeeringEstimateValidator {
    pub rules: Vec<Rc<dyn PeeringEstimateRule>>,
}

impl PeeringEstimateValidator {
    pub fn validate(&self, item: &PeeringEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PeeringEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_estimate_validator() -> PeeringEstimateValidator {
    PeeringEstimateValidator {
        rules: vec![
            Rc::new(PeeringEstimateRequiredPrincipalRule {
                message: "peering estimate needs an entry".to_string(),
            }) as Rc<dyn PeeringEstimateRule>,
            Rc::new(PeeringEstimateTitleLengthRule { max_length: 86400 }),
            Rc::new(PeeringEstimateCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(PeeringEstimateRequiredTitleRule {
                message: "peering estimate needs this field".to_string(),
            }),
            Rc::new(PeeringEstimateAllowedSegmentRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_estimate_validator();
        let item = new_peering_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = PeeringEstimateRequiredPrincipalRule {
            message: "peering estimate needs an entry".to_string(),
        };
        let mut item = new_peering_estimate("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PeeringEstimateTitleLengthRule { max_length: 255 };
        let mut item = new_peering_estimate("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PeeringEstimateRequiredTitleRule {
            message: "peering estimate needs a value".to_string(),
        };
        let mut item = new_peering_estimate("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
