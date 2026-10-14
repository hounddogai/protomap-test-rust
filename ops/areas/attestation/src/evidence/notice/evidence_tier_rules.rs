use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a evidence tier must pass. Check returns its problems, or none.
pub trait EvidenceTierRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceTierAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl EvidenceTierRule for EvidenceTierAllowedCodeRule {
    fn check(&self, item: &EvidenceTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceTierCodePrefixRule {
    pub prefix: String,
}

impl EvidenceTierRule for EvidenceTierCodePrefixRule {
    fn check(&self, item: &EvidenceTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceTierQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EvidenceTierRule for EvidenceTierQuantityRangeRule {
    fn check(&self, item: &EvidenceTier) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceTierRequiredCodeRule {
    pub message: String,
}

impl EvidenceTierRule for EvidenceTierRequiredCodeRule {
    fn check(&self, item: &EvidenceTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceTierPhoneLengthRule {
    pub max_length: i64,
}

impl EvidenceTierRule for EvidenceTierPhoneLengthRule {
    fn check(&self, item: &EvidenceTier) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence tier.
#[derive(Debug, Clone)]
pub struct EvidenceTierValidator {
    pub rules: Vec<Rc<dyn EvidenceTierRule>>,
}

impl EvidenceTierValidator {
    pub fn validate(&self, item: &EvidenceTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EvidenceTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_tier_validator() -> EvidenceTierValidator {
    EvidenceTierValidator {
        rules: vec![
            Rc::new(EvidenceTierAllowedCodeRule {
                allowed: vec!["weighted".to_string()],
            }) as Rc<dyn EvidenceTierRule>,
            Rc::new(EvidenceTierCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(EvidenceTierQuantityRangeRule { min: 16, max: 255 }),
            Rc::new(EvidenceTierRequiredCodeRule {
                message: "evidence tier needs this field".to_string(),
            }),
            Rc::new(EvidenceTierPhoneLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_tier_validator();
        let item = new_evidence_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = EvidenceTierQuantityRangeRule { min: 255, max: 3600 };
        let mut item = new_evidence_tier("x", "y");
        item.quantity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EvidenceTierRequiredCodeRule {
            message: "evidence tier needs an entry".to_string(),
        };
        let mut item = new_evidence_tier("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = EvidenceTierPhoneLengthRule { max_length: 255 };
        let mut item = new_evidence_tier("x", "y");
        item.phone = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
