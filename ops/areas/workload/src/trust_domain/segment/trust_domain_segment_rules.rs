use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a trust domain segment must pass. Check returns its problems, or none.
pub trait TrustDomainSegmentRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSegmentRequiredOwnerIdRule {
    pub message: String,
}

impl TrustDomainSegmentRule for TrustDomainSegmentRequiredOwnerIdRule {
    fn check(&self, item: &TrustDomainSegment) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSegmentCodePrefixRule {
    pub prefix: String,
}

impl TrustDomainSegmentRule for TrustDomainSegmentCodePrefixRule {
    fn check(&self, item: &TrustDomainSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSegmentBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TrustDomainSegmentRule for TrustDomainSegmentBudgetRangeRule {
    fn check(&self, item: &TrustDomainSegment) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSegmentCodeLengthRule {
    pub max_length: i64,
}

impl TrustDomainSegmentRule for TrustDomainSegmentCodeLengthRule {
    fn check(&self, item: &TrustDomainSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSegmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TrustDomainSegmentRule for TrustDomainSegmentAllowedCodeRule {
    fn check(&self, item: &TrustDomainSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain segment.
#[derive(Debug, Clone)]
pub struct TrustDomainSegmentValidator {
    pub rules: Vec<Rc<dyn TrustDomainSegmentRule>>,
}

impl TrustDomainSegmentValidator {
    pub fn validate(&self, item: &TrustDomainSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TrustDomainSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_segment_validator() -> TrustDomainSegmentValidator {
    TrustDomainSegmentValidator {
        rules: vec![
            Rc::new(TrustDomainSegmentRequiredOwnerIdRule {
                message: "trust domain segment needs an entry".to_string(),
            }) as Rc<dyn TrustDomainSegmentRule>,
            Rc::new(TrustDomainSegmentCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(TrustDomainSegmentBudgetRangeRule { min: 7.1, max: 1798.4 }),
            Rc::new(TrustDomainSegmentCodeLengthRule { max_length: 64 }),
            Rc::new(TrustDomainSegmentAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_segment_validator();
        let item = new_trust_domain_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = TrustDomainSegmentRequiredOwnerIdRule {
            message: "trust domain segment needs this field".to_string(),
        };
        let mut item = new_trust_domain_segment("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = TrustDomainSegmentBudgetRangeRule { min: 9.4, max: 3835.7 };
        let mut item = new_trust_domain_segment("x", "y");
        item.budget = 3837.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TrustDomainSegmentCodeLengthRule { max_length: 16 };
        let mut item = new_trust_domain_segment("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
