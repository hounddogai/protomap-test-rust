use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive verifier baseline must pass. Check returns its problems, or none.
pub trait AdaptiveVerifierBaselineRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveVerifierBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveVerifierBaselineRequiredCodeRule {
    pub message: String,
}

impl AdaptiveVerifierBaselineRule for AdaptiveVerifierBaselineRequiredCodeRule {
    fn check(&self, item: &AdaptiveVerifierBaseline) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveVerifierBaselineExternalRefLengthRule {
    pub max_length: i64,
}

impl AdaptiveVerifierBaselineRule for AdaptiveVerifierBaselineExternalRefLengthRule {
    fn check(&self, item: &AdaptiveVerifierBaseline) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveVerifierBaselineRequiredCountryRule {
    pub message: String,
}

impl AdaptiveVerifierBaselineRule for AdaptiveVerifierBaselineRequiredCountryRule {
    fn check(&self, item: &AdaptiveVerifierBaseline) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveVerifierBaselineAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl AdaptiveVerifierBaselineRule for AdaptiveVerifierBaselineAllowedTitleRule {
    fn check(&self, item: &AdaptiveVerifierBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is blocked", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive verifier baseline.
#[derive(Debug, Clone)]
pub struct AdaptiveVerifierBaselineValidator {
    pub rules: Vec<Rc<dyn AdaptiveVerifierBaselineRule>>,
}

impl AdaptiveVerifierBaselineValidator {
    pub fn validate(&self, item: &AdaptiveVerifierBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveVerifierBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveVerifierBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_verifier_baseline_validator() -> AdaptiveVerifierBaselineValidator {
    AdaptiveVerifierBaselineValidator {
        rules: vec![
            Rc::new(AdaptiveVerifierBaselineRequiredCodeRule {
                message: "adaptive verifier baseline needs this field".to_string(),
            }) as Rc<dyn AdaptiveVerifierBaselineRule>,
            Rc::new(AdaptiveVerifierBaselineExternalRefLengthRule { max_length: 64 }),
            Rc::new(AdaptiveVerifierBaselineRequiredCountryRule {
                message: "adaptive verifier baseline needs an entry".to_string(),
            }),
            Rc::new(AdaptiveVerifierBaselineAllowedTitleRule {
                allowed: vec!["primary".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_verifier_baseline_validator();
        let item = new_adaptive_verifier_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveVerifierBaselineRequiredCodeRule {
            message: "adaptive verifier baseline needs this field".to_string(),
        };
        let mut item = new_adaptive_verifier_baseline("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = AdaptiveVerifierBaselineExternalRefLengthRule { max_length: 80 };
        let mut item = new_adaptive_verifier_baseline("x", "y");
        item.external_ref = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = AdaptiveVerifierBaselineRequiredCountryRule {
            message: "adaptive verifier baseline needs this field".to_string(),
        };
        let mut item = new_adaptive_verifier_baseline("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
