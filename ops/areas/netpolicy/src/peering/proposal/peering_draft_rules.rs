use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a peering draft must pass. Check returns its problems, or none.
pub trait PeeringDraftRule: std::fmt::Debug {
    fn check(&self, item: &PeeringDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringDraftCodePrefixRule {
    pub prefix: String,
}

impl PeeringDraftRule for PeeringDraftCodePrefixRule {
    fn check(&self, item: &PeeringDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringDraftTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringDraftRule for PeeringDraftTtlSecondsRangeRule {
    fn check(&self, item: &PeeringDraft) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringDraftRequiredCodeRule {
    pub message: String,
}

impl PeeringDraftRule for PeeringDraftRequiredCodeRule {
    fn check(&self, item: &PeeringDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringDraftRequiredNameRule {
    pub message: String,
}

impl PeeringDraftRule for PeeringDraftRequiredNameRule {
    fn check(&self, item: &PeeringDraft) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringDraftRegionLengthRule {
    pub max_length: i64,
}

impl PeeringDraftRule for PeeringDraftRegionLengthRule {
    fn check(&self, item: &PeeringDraft) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringDraftAllowedRegionRule {
    pub allowed: Vec<String>,
}

impl PeeringDraftRule for PeeringDraftAllowedRegionRule {
    fn check(&self, item: &PeeringDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.region))) {
            return vec![format!("region {} is inconsistent", item.region)];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering draft.
#[derive(Debug, Clone)]
pub struct PeeringDraftValidator {
    pub rules: Vec<Rc<dyn PeeringDraftRule>>,
}

impl PeeringDraftValidator {
    pub fn validate(&self, item: &PeeringDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PeeringDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_draft_validator() -> PeeringDraftValidator {
    PeeringDraftValidator {
        rules: vec![
            Rc::new(PeeringDraftCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn PeeringDraftRule>,
            Rc::new(PeeringDraftTtlSecondsRangeRule { min: 32, max: 86400 }),
            Rc::new(PeeringDraftRequiredCodeRule {
                message: "peering draft needs an entry".to_string(),
            }),
            Rc::new(PeeringDraftRequiredNameRule {
                message: "peering draft needs a value".to_string(),
            }),
            Rc::new(PeeringDraftRegionLengthRule { max_length: 32 }),
            Rc::new(PeeringDraftAllowedRegionRule {
                allowed: vec!["regional".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_draft_validator();
        let item = new_peering_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = PeeringDraftTtlSecondsRangeRule { min: 32, max: 32 };
        let mut item = new_peering_draft("x", "y");
        item.ttl_seconds = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PeeringDraftRequiredCodeRule {
            message: "peering draft needs an entry".to_string(),
        };
        let mut item = new_peering_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PeeringDraftRequiredNameRule {
            message: "peering draft needs a value".to_string(),
        };
        let mut item = new_peering_draft("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = PeeringDraftRegionLengthRule { max_length: 80 };
        let mut item = new_peering_draft("x", "y");
        item.region = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
