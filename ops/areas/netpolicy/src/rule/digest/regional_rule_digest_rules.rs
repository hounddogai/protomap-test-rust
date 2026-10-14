use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional rule digest must pass. Check returns its problems, or none.
pub trait RegionalRuleDigestRule: std::fmt::Debug {
    fn check(&self, item: &RegionalRuleDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRuleDigestNameLengthRule {
    pub max_length: i64,
}

impl RegionalRuleDigestRule for RegionalRuleDigestNameLengthRule {
    fn check(&self, item: &RegionalRuleDigest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRuleDigestRequiredAudienceRule {
    pub message: String,
}

impl RegionalRuleDigestRule for RegionalRuleDigestRequiredAudienceRule {
    fn check(&self, item: &RegionalRuleDigest) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRuleDigestDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalRuleDigestRule for RegionalRuleDigestDiscountRangeRule {
    fn check(&self, item: &RegionalRuleDigest) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRuleDigestCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalRuleDigestRule for RegionalRuleDigestCeilingRangeRule {
    fn check(&self, item: &RegionalRuleDigest) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRuleDigestRequiredCodeRule {
    pub message: String,
}

impl RegionalRuleDigestRule for RegionalRuleDigestRequiredCodeRule {
    fn check(&self, item: &RegionalRuleDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRuleDigestAudienceLengthRule {
    pub max_length: i64,
}

impl RegionalRuleDigestRule for RegionalRuleDigestAudienceLengthRule {
    fn check(&self, item: &RegionalRuleDigest) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional rule digest.
#[derive(Debug, Clone)]
pub struct RegionalRuleDigestValidator {
    pub rules: Vec<Rc<dyn RegionalRuleDigestRule>>,
}

impl RegionalRuleDigestValidator {
    pub fn validate(&self, item: &RegionalRuleDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalRuleDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalRuleDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_rule_digest_validator() -> RegionalRuleDigestValidator {
    RegionalRuleDigestValidator {
        rules: vec![
            Rc::new(RegionalRuleDigestNameLengthRule { max_length: 255 }) as Rc<dyn RegionalRuleDigestRule>,
            Rc::new(RegionalRuleDigestRequiredAudienceRule {
                message: "regional rule digest needs a value".to_string(),
            }),
            Rc::new(RegionalRuleDigestDiscountRangeRule { min: 4.0, max: 1802.2 }),
            Rc::new(RegionalRuleDigestCeilingRangeRule { min: 0.5, max: 2027.4 }),
            Rc::new(RegionalRuleDigestRequiredCodeRule {
                message: "regional rule digest needs a value".to_string(),
            }),
            Rc::new(RegionalRuleDigestAudienceLengthRule { max_length: 16 }),
        ],
    }
}
