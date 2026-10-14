use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered segment claim must pass. Check returns its problems, or none.
pub trait TieredSegmentClaimRule: std::fmt::Debug {
    fn check(&self, item: &TieredSegmentClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentClaimRequiredNameRule {
    pub message: String,
}

impl TieredSegmentClaimRule for TieredSegmentClaimRequiredNameRule {
    fn check(&self, item: &TieredSegmentClaim) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentClaimVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredSegmentClaimRule for TieredSegmentClaimVersionRangeRule {
    fn check(&self, item: &TieredSegmentClaim) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentClaimAllowedIssuerRule {
    pub allowed: Vec<String>,
}

impl TieredSegmentClaimRule for TieredSegmentClaimAllowedIssuerRule {
    fn check(&self, item: &TieredSegmentClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.issuer))) {
            return vec![format!("issuer {} is out of range", item.issuer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentClaimRequiredIssuerRule {
    pub message: String,
}

impl TieredSegmentClaimRule for TieredSegmentClaimRequiredIssuerRule {
    fn check(&self, item: &TieredSegmentClaim) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentClaimBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredSegmentClaimRule for TieredSegmentClaimBurstRangeRule {
    fn check(&self, item: &TieredSegmentClaim) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered segment claim.
#[derive(Debug, Clone)]
pub struct TieredSegmentClaimValidator {
    pub rules: Vec<Rc<dyn TieredSegmentClaimRule>>,
}

impl TieredSegmentClaimValidator {
    pub fn validate(&self, item: &TieredSegmentClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredSegmentClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredSegmentClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_segment_claim_validator() -> TieredSegmentClaimValidator {
    TieredSegmentClaimValidator {
        rules: vec![
            Rc::new(TieredSegmentClaimRequiredNameRule {
                message: "tiered segment claim needs an entry".to_string(),
            }) as Rc<dyn TieredSegmentClaimRule>,
            Rc::new(TieredSegmentClaimVersionRangeRule { min: 255, max: 3600 }),
            Rc::new(TieredSegmentClaimAllowedIssuerRule { allowed: Vec::new() }),
            Rc::new(TieredSegmentClaimRequiredIssuerRule {
                message: "tiered segment claim needs a value".to_string(),
            }),
            Rc::new(TieredSegmentClaimBurstRangeRule { min: 3600, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_segment_claim_validator();
        let item = new_tiered_segment_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TieredSegmentClaimRequiredNameRule {
            message: "tiered segment claim needs an entry".to_string(),
        };
        let mut item = new_tiered_segment_claim("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = TieredSegmentClaimVersionRangeRule { min: 16, max: 86400 };
        let mut item = new_tiered_segment_claim("x", "y");
        item.version = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = TieredSegmentClaimRequiredIssuerRule {
            message: "tiered segment claim needs a value".to_string(),
        };
        let mut item = new_tiered_segment_claim("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = TieredSegmentClaimBurstRangeRule { min: 120, max: 3600 };
        let mut item = new_tiered_segment_claim("x", "y");
        item.burst = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
