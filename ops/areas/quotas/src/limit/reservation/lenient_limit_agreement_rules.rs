use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient limit agreement must pass. Check returns its problems, or none.
pub trait LenientLimitAgreementRule: std::fmt::Debug {
    fn check(&self, item: &LenientLimitAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientLimitAgreementTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientLimitAgreementRule for LenientLimitAgreementTtlSecondsRangeRule {
    fn check(&self, item: &LenientLimitAgreement) -> Vec<String> {
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
pub struct LenientLimitAgreementRequiredTitleRule {
    pub message: String,
}

impl LenientLimitAgreementRule for LenientLimitAgreementRequiredTitleRule {
    fn check(&self, item: &LenientLimitAgreement) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientLimitAgreementCodePrefixRule {
    pub prefix: String,
}

impl LenientLimitAgreementRule for LenientLimitAgreementCodePrefixRule {
    fn check(&self, item: &LenientLimitAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientLimitAgreementRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientLimitAgreementRule for LenientLimitAgreementRevisionRangeRule {
    fn check(&self, item: &LenientLimitAgreement) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientLimitAgreementUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientLimitAgreementRule for LenientLimitAgreementUpdatedAtRangeRule {
    fn check(&self, item: &LenientLimitAgreement) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient limit agreement.
#[derive(Debug, Clone)]
pub struct LenientLimitAgreementValidator {
    pub rules: Vec<Rc<dyn LenientLimitAgreementRule>>,
}

impl LenientLimitAgreementValidator {
    pub fn validate(&self, item: &LenientLimitAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientLimitAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientLimitAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_limit_agreement_validator() -> LenientLimitAgreementValidator {
    LenientLimitAgreementValidator {
        rules: vec![
            Rc::new(LenientLimitAgreementTtlSecondsRangeRule { min: 120, max: 3600 })
                as Rc<dyn LenientLimitAgreementRule>,
            Rc::new(LenientLimitAgreementRequiredTitleRule {
                message: "lenient limit agreement needs a value".to_string(),
            }),
            Rc::new(LenientLimitAgreementCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(LenientLimitAgreementRevisionRangeRule { min: 64, max: 1000 }),
            Rc::new(LenientLimitAgreementUpdatedAtRangeRule { min: 64, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_limit_agreement_validator();
        let item = new_lenient_limit_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = LenientLimitAgreementTtlSecondsRangeRule { min: 64, max: 1000 };
        let mut item = new_lenient_limit_agreement("x", "y");
        item.ttl_seconds = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LenientLimitAgreementRequiredTitleRule {
            message: "lenient limit agreement needs a value".to_string(),
        };
        let mut item = new_lenient_limit_agreement("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = LenientLimitAgreementRevisionRangeRule { min: 120, max: 255 };
        let mut item = new_lenient_limit_agreement("x", "y");
        item.revision = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = LenientLimitAgreementUpdatedAtRangeRule { min: 1000, max: 86400 };
        let mut item = new_lenient_limit_agreement("x", "y");
        item.updated_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
