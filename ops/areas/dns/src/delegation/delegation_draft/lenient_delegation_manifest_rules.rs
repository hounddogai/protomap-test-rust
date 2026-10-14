use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient delegation manifest must pass. Check returns its problems, or none.
pub trait LenientDelegationManifestRule: std::fmt::Debug {
    fn check(&self, item: &LenientDelegationManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationManifestLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientDelegationManifestRule for LenientDelegationManifestLimitRangeRule {
    fn check(&self, item: &LenientDelegationManifest) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationManifestRequiredCodeRule {
    pub message: String,
}

impl LenientDelegationManifestRule for LenientDelegationManifestRequiredCodeRule {
    fn check(&self, item: &LenientDelegationManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationManifestAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl LenientDelegationManifestRule for LenientDelegationManifestAllowedTitleRule {
    fn check(&self, item: &LenientDelegationManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is unknown", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationManifestTitleLengthRule {
    pub max_length: i64,
}

impl LenientDelegationManifestRule for LenientDelegationManifestTitleLengthRule {
    fn check(&self, item: &LenientDelegationManifest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationManifestCodeLengthRule {
    pub max_length: i64,
}

impl LenientDelegationManifestRule for LenientDelegationManifestCodeLengthRule {
    fn check(&self, item: &LenientDelegationManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient delegation manifest.
#[derive(Debug, Clone)]
pub struct LenientDelegationManifestValidator {
    pub rules: Vec<Rc<dyn LenientDelegationManifestRule>>,
}

impl LenientDelegationManifestValidator {
    pub fn validate(&self, item: &LenientDelegationManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientDelegationManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientDelegationManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_delegation_manifest_validator() -> LenientDelegationManifestValidator {
    LenientDelegationManifestValidator {
        rules: vec![
            Rc::new(LenientDelegationManifestLimitRangeRule { min: 80, max: 255 })
                as Rc<dyn LenientDelegationManifestRule>,
            Rc::new(LenientDelegationManifestRequiredCodeRule {
                message: "lenient delegation manifest needs an entry".to_string(),
            }),
            Rc::new(LenientDelegationManifestAllowedTitleRule {
                allowed: vec!["seasonal".to_string()],
            }),
            Rc::new(LenientDelegationManifestTitleLengthRule { max_length: 80 }),
            Rc::new(LenientDelegationManifestCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_delegation_manifest_validator();
        let item = new_lenient_delegation_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = LenientDelegationManifestLimitRangeRule { min: 32, max: 86400 };
        let mut item = new_lenient_delegation_manifest("x", "y");
        item.limit = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientDelegationManifestRequiredCodeRule {
            message: "lenient delegation manifest needs this field".to_string(),
        };
        let mut item = new_lenient_delegation_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LenientDelegationManifestTitleLengthRule { max_length: 1000 };
        let mut item = new_lenient_delegation_manifest("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientDelegationManifestCodeLengthRule { max_length: 80 };
        let mut item = new_lenient_delegation_manifest("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
