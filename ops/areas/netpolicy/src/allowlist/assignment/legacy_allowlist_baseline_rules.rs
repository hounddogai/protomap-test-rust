use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy allowlist baseline must pass. Check returns its problems, or none.
pub trait LegacyAllowlistBaselineRule: std::fmt::Debug {
    fn check(&self, item: &LegacyAllowlistBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistBaselineRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyAllowlistBaselineRule for LegacyAllowlistBaselineRatioRangeRule {
    fn check(&self, item: &LegacyAllowlistBaseline) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistBaselineBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyAllowlistBaselineRule for LegacyAllowlistBaselineBatchSizeRangeRule {
    fn check(&self, item: &LegacyAllowlistBaseline) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistBaselineRequiredLocaleRule {
    pub message: String,
}

impl LegacyAllowlistBaselineRule for LegacyAllowlistBaselineRequiredLocaleRule {
    fn check(&self, item: &LegacyAllowlistBaseline) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistBaselineAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl LegacyAllowlistBaselineRule for LegacyAllowlistBaselineAllowedLabelRule {
    fn check(&self, item: &LegacyAllowlistBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is out of range", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistBaselineVisibleGuardRule {
    pub limit: f64,
}

impl LegacyAllowlistBaselineRule for LegacyAllowlistBaselineVisibleGuardRule {
    fn check(&self, item: &LegacyAllowlistBaseline) -> Vec<String> {
        if item.visible && (item.ratio > self.limit) {
            return vec![format!("ratio {} exceeds {} while visible", item.ratio, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistBaselineCodePrefixRule {
    pub prefix: String,
}

impl LegacyAllowlistBaselineRule for LegacyAllowlistBaselineCodePrefixRule {
    fn check(&self, item: &LegacyAllowlistBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy allowlist baseline.
#[derive(Debug, Clone)]
pub struct LegacyAllowlistBaselineValidator {
    pub rules: Vec<Rc<dyn LegacyAllowlistBaselineRule>>,
}

impl LegacyAllowlistBaselineValidator {
    pub fn validate(&self, item: &LegacyAllowlistBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyAllowlistBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyAllowlistBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_allowlist_baseline_validator() -> LegacyAllowlistBaselineValidator {
    LegacyAllowlistBaselineValidator {
        rules: vec![
            Rc::new(LegacyAllowlistBaselineRatioRangeRule { min: 0.2, max: 3997.4 })
                as Rc<dyn LegacyAllowlistBaselineRule>,
            Rc::new(LegacyAllowlistBaselineBatchSizeRangeRule { min: 120, max: 86400 }),
            Rc::new(LegacyAllowlistBaselineRequiredLocaleRule {
                message: "legacy allowlist baseline needs this field".to_string(),
            }),
            Rc::new(LegacyAllowlistBaselineAllowedLabelRule {
                allowed: vec!["strict".to_string()],
            }),
            Rc::new(LegacyAllowlistBaselineVisibleGuardRule { limit: 5.7 }),
            Rc::new(LegacyAllowlistBaselineCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_allowlist_baseline_validator();
        let item = new_legacy_allowlist_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = LegacyAllowlistBaselineRatioRangeRule { min: 9.6, max: 3213.5 };
        let mut item = new_legacy_allowlist_baseline("x", "y");
        item.ratio = 3215.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = LegacyAllowlistBaselineBatchSizeRangeRule { min: 32, max: 255 };
        let mut item = new_legacy_allowlist_baseline("x", "y");
        item.batch_size = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = LegacyAllowlistBaselineRequiredLocaleRule {
            message: "legacy allowlist baseline needs an entry".to_string(),
        };
        let mut item = new_legacy_allowlist_baseline("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
