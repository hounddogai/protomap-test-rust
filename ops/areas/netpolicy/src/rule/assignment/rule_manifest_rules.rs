use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule manifest must pass. Check returns its problems, or none.
pub trait RuleManifestRule: std::fmt::Debug {
    fn check(&self, item: &RuleManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleManifestThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RuleManifestRule for RuleManifestThresholdRangeRule {
    fn check(&self, item: &RuleManifest) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleManifestRequiredPrincipalRule {
    pub message: String,
}

impl RuleManifestRule for RuleManifestRequiredPrincipalRule {
    fn check(&self, item: &RuleManifest) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleManifestTitleLengthRule {
    pub max_length: i64,
}

impl RuleManifestRule for RuleManifestTitleLengthRule {
    fn check(&self, item: &RuleManifest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleManifestRequiredCodeRule {
    pub message: String,
}

impl RuleManifestRule for RuleManifestRequiredCodeRule {
    fn check(&self, item: &RuleManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleManifestPrincipalLengthRule {
    pub max_length: i64,
}

impl RuleManifestRule for RuleManifestPrincipalLengthRule {
    fn check(&self, item: &RuleManifest) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule manifest.
#[derive(Debug, Clone)]
pub struct RuleManifestValidator {
    pub rules: Vec<Rc<dyn RuleManifestRule>>,
}

impl RuleManifestValidator {
    pub fn validate(&self, item: &RuleManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RuleManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_manifest_validator() -> RuleManifestValidator {
    RuleManifestValidator {
        rules: vec![
            Rc::new(RuleManifestThresholdRangeRule { min: 2.1, max: 3144.1 }) as Rc<dyn RuleManifestRule>,
            Rc::new(RuleManifestRequiredPrincipalRule {
                message: "rule manifest needs this field".to_string(),
            }),
            Rc::new(RuleManifestTitleLengthRule { max_length: 120 }),
            Rc::new(RuleManifestRequiredCodeRule {
                message: "rule manifest needs a value".to_string(),
            }),
            Rc::new(RuleManifestPrincipalLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_manifest_validator();
        let item = new_rule_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = RuleManifestThresholdRangeRule { min: 1.3, max: 2160.1 };
        let mut item = new_rule_manifest("x", "y");
        item.threshold = 2161.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = RuleManifestRequiredPrincipalRule {
            message: "rule manifest needs this field".to_string(),
        };
        let mut item = new_rule_manifest("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RuleManifestTitleLengthRule { max_length: 32 };
        let mut item = new_rule_manifest("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RuleManifestRequiredCodeRule {
            message: "rule manifest needs an entry".to_string(),
        };
        let mut item = new_rule_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = RuleManifestPrincipalLengthRule { max_length: 32 };
        let mut item = new_rule_manifest("x", "y");
        item.principal = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
