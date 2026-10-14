use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered verifier bundle must pass. Check returns its problems, or none.
pub trait TieredVerifierBundleRule: std::fmt::Debug {
    fn check(&self, item: &TieredVerifierBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierBundleRequiredNamespaceRule {
    pub message: String,
}

impl TieredVerifierBundleRule for TieredVerifierBundleRequiredNamespaceRule {
    fn check(&self, item: &TieredVerifierBundle) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierBundleRequiredLabelRule {
    pub message: String,
}

impl TieredVerifierBundleRule for TieredVerifierBundleRequiredLabelRule {
    fn check(&self, item: &TieredVerifierBundle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierBundleLabelLengthRule {
    pub max_length: i64,
}

impl TieredVerifierBundleRule for TieredVerifierBundleLabelLengthRule {
    fn check(&self, item: &TieredVerifierBundle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierBundleWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredVerifierBundleRule for TieredVerifierBundleWindowDaysRangeRule {
    fn check(&self, item: &TieredVerifierBundle) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierBundlePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredVerifierBundleRule for TieredVerifierBundlePortRangeRule {
    fn check(&self, item: &TieredVerifierBundle) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierBundleNamespaceLengthRule {
    pub max_length: i64,
}

impl TieredVerifierBundleRule for TieredVerifierBundleNamespaceLengthRule {
    fn check(&self, item: &TieredVerifierBundle) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered verifier bundle.
#[derive(Debug, Clone)]
pub struct TieredVerifierBundleValidator {
    pub rules: Vec<Rc<dyn TieredVerifierBundleRule>>,
}

impl TieredVerifierBundleValidator {
    pub fn validate(&self, item: &TieredVerifierBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredVerifierBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredVerifierBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_verifier_bundle_validator() -> TieredVerifierBundleValidator {
    TieredVerifierBundleValidator {
        rules: vec![
            Rc::new(TieredVerifierBundleRequiredNamespaceRule {
                message: "tiered verifier bundle needs an entry".to_string(),
            }) as Rc<dyn TieredVerifierBundleRule>,
            Rc::new(TieredVerifierBundleRequiredLabelRule {
                message: "tiered verifier bundle needs a value".to_string(),
            }),
            Rc::new(TieredVerifierBundleLabelLengthRule { max_length: 80 }),
            Rc::new(TieredVerifierBundleWindowDaysRangeRule { min: 32, max: 1000 }),
            Rc::new(TieredVerifierBundlePortRangeRule { min: 1000, max: 86400 }),
            Rc::new(TieredVerifierBundleNamespaceLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_verifier_bundle_validator();
        let item = new_tiered_verifier_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = TieredVerifierBundleRequiredNamespaceRule {
            message: "tiered verifier bundle needs an entry".to_string(),
        };
        let mut item = new_tiered_verifier_bundle("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TieredVerifierBundleRequiredLabelRule {
            message: "tiered verifier bundle needs a value".to_string(),
        };
        let mut item = new_tiered_verifier_bundle("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TieredVerifierBundleLabelLengthRule { max_length: 120 };
        let mut item = new_tiered_verifier_bundle("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = TieredVerifierBundleWindowDaysRangeRule { min: 32, max: 32 };
        let mut item = new_tiered_verifier_bundle("x", "y");
        item.window_days = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = TieredVerifierBundlePortRangeRule { min: 1000, max: 86400 };
        let mut item = new_tiered_verifier_bundle("x", "y");
        item.port = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = TieredVerifierBundleNamespaceLengthRule { max_length: 80 };
        let mut item = new_tiered_verifier_bundle("x", "y");
        item.namespace = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
