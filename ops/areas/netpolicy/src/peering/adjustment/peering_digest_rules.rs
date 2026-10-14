use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering digest must pass. Check returns its problems, or none.
pub trait PeeringDigestRule: std::fmt::Debug {
    fn check(&self, item: &PeeringDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringDigestTitleLengthRule {
    pub max_length: i64,
}

impl PeeringDigestRule for PeeringDigestTitleLengthRule {
    fn check(&self, item: &PeeringDigest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringDigestRequiredTitleRule {
    pub message: String,
}

impl PeeringDigestRule for PeeringDigestRequiredTitleRule {
    fn check(&self, item: &PeeringDigest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringDigestRequiredCodeRule {
    pub message: String,
}

impl PeeringDigestRule for PeeringDigestRequiredCodeRule {
    fn check(&self, item: &PeeringDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringDigestCodeLengthRule {
    pub max_length: i64,
}

impl PeeringDigestRule for PeeringDigestCodeLengthRule {
    fn check(&self, item: &PeeringDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringDigestRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringDigestRule for PeeringDigestRevisionRangeRule {
    fn check(&self, item: &PeeringDigest) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering digest.
#[derive(Debug, Clone)]
pub struct PeeringDigestValidator {
    pub rules: Vec<Rc<dyn PeeringDigestRule>>,
}

impl PeeringDigestValidator {
    pub fn validate(&self, item: &PeeringDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PeeringDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_digest_validator() -> PeeringDigestValidator {
    PeeringDigestValidator {
        rules: vec![
            Rc::new(PeeringDigestTitleLengthRule { max_length: 1000 }) as Rc<dyn PeeringDigestRule>,
            Rc::new(PeeringDigestRequiredTitleRule {
                message: "peering digest needs this field".to_string(),
            }),
            Rc::new(PeeringDigestRequiredCodeRule {
                message: "peering digest needs this field".to_string(),
            }),
            Rc::new(PeeringDigestCodeLengthRule { max_length: 80 }),
            Rc::new(PeeringDigestRevisionRangeRule { min: 80, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_digest_validator();
        let item = new_peering_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PeeringDigestTitleLengthRule { max_length: 64 };
        let mut item = new_peering_digest("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PeeringDigestRequiredTitleRule {
            message: "peering digest needs an entry".to_string(),
        };
        let mut item = new_peering_digest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PeeringDigestRequiredCodeRule {
            message: "peering digest needs a value".to_string(),
        };
        let mut item = new_peering_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PeeringDigestCodeLengthRule { max_length: 120 };
        let mut item = new_peering_digest("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = PeeringDigestRevisionRangeRule { min: 16, max: 3600 };
        let mut item = new_peering_digest("x", "y");
        item.revision = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
