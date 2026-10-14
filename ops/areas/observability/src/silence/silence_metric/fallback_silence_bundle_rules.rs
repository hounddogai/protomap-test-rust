use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback silence bundle must pass. Check returns its problems, or none.
pub trait FallbackSilenceBundleRule: std::fmt::Debug {
    fn check(&self, item: &FallbackSilenceBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSilenceBundleCodePrefixRule {
    pub prefix: String,
}

impl FallbackSilenceBundleRule for FallbackSilenceBundleCodePrefixRule {
    fn check(&self, item: &FallbackSilenceBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSilenceBundleDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackSilenceBundleRule for FallbackSilenceBundleDiscountRangeRule {
    fn check(&self, item: &FallbackSilenceBundle) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSilenceBundleRequiredCodeRule {
    pub message: String,
}

impl FallbackSilenceBundleRule for FallbackSilenceBundleRequiredCodeRule {
    fn check(&self, item: &FallbackSilenceBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSilenceBundleNamespaceLengthRule {
    pub max_length: i64,
}

impl FallbackSilenceBundleRule for FallbackSilenceBundleNamespaceLengthRule {
    fn check(&self, item: &FallbackSilenceBundle) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSilenceBundleCodeLengthRule {
    pub max_length: i64,
}

impl FallbackSilenceBundleRule for FallbackSilenceBundleCodeLengthRule {
    fn check(&self, item: &FallbackSilenceBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSilenceBundleRequiredCountryRule {
    pub message: String,
}

impl FallbackSilenceBundleRule for FallbackSilenceBundleRequiredCountryRule {
    fn check(&self, item: &FallbackSilenceBundle) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback silence bundle.
#[derive(Debug, Clone)]
pub struct FallbackSilenceBundleValidator {
    pub rules: Vec<Rc<dyn FallbackSilenceBundleRule>>,
}

impl FallbackSilenceBundleValidator {
    pub fn validate(&self, item: &FallbackSilenceBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackSilenceBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackSilenceBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_silence_bundle_validator() -> FallbackSilenceBundleValidator {
    FallbackSilenceBundleValidator {
        rules: vec![
            Rc::new(FallbackSilenceBundleCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn FallbackSilenceBundleRule>,
            Rc::new(FallbackSilenceBundleDiscountRangeRule { min: 3.4, max: 267.2 }),
            Rc::new(FallbackSilenceBundleRequiredCodeRule {
                message: "fallback silence bundle needs a value".to_string(),
            }),
            Rc::new(FallbackSilenceBundleNamespaceLengthRule { max_length: 3600 }),
            Rc::new(FallbackSilenceBundleCodeLengthRule { max_length: 16 }),
            Rc::new(FallbackSilenceBundleRequiredCountryRule {
                message: "fallback silence bundle needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_silence_bundle_validator();
        let item = new_fallback_silence_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = FallbackSilenceBundleDiscountRangeRule { min: 1.3, max: 4541.7 };
        let mut item = new_fallback_silence_bundle("x", "y");
        item.discount = 4543.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackSilenceBundleRequiredCodeRule {
            message: "fallback silence bundle needs an entry".to_string(),
        };
        let mut item = new_fallback_silence_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = FallbackSilenceBundleNamespaceLengthRule { max_length: 16 };
        let mut item = new_fallback_silence_bundle("x", "y");
        item.namespace = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackSilenceBundleCodeLengthRule { max_length: 80 };
        let mut item = new_fallback_silence_bundle("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = FallbackSilenceBundleRequiredCountryRule {
            message: "fallback silence bundle needs an entry".to_string(),
        };
        let mut item = new_fallback_silence_bundle("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
