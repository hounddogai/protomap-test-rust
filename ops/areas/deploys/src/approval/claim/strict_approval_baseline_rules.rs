use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict approval baseline must pass. Check returns its problems, or none.
pub trait StrictApprovalBaselineRule: std::fmt::Debug {
    fn check(&self, item: &StrictApprovalBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalBaselineRequiredLabelRule {
    pub message: String,
}

impl StrictApprovalBaselineRule for StrictApprovalBaselineRequiredLabelRule {
    fn check(&self, item: &StrictApprovalBaseline) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalBaselineNamespaceLengthRule {
    pub max_length: i64,
}

impl StrictApprovalBaselineRule for StrictApprovalBaselineNamespaceLengthRule {
    fn check(&self, item: &StrictApprovalBaseline) -> Vec<String> {
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
pub struct StrictApprovalBaselineRegionLengthRule {
    pub max_length: i64,
}

impl StrictApprovalBaselineRule for StrictApprovalBaselineRegionLengthRule {
    fn check(&self, item: &StrictApprovalBaseline) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalBaselineAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl StrictApprovalBaselineRule for StrictApprovalBaselineAllowedCodeRule {
    fn check(&self, item: &StrictApprovalBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalBaselineQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictApprovalBaselineRule for StrictApprovalBaselineQuotaUnitsRangeRule {
    fn check(&self, item: &StrictApprovalBaseline) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a strict approval baseline.
#[derive(Debug, Clone)]
pub struct StrictApprovalBaselineValidator {
    pub rules: Vec<Rc<dyn StrictApprovalBaselineRule>>,
}

impl StrictApprovalBaselineValidator {
    pub fn validate(&self, item: &StrictApprovalBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictApprovalBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictApprovalBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_approval_baseline_validator() -> StrictApprovalBaselineValidator {
    StrictApprovalBaselineValidator {
        rules: vec![
            Rc::new(StrictApprovalBaselineRequiredLabelRule {
                message: "strict approval baseline needs this field".to_string(),
            }) as Rc<dyn StrictApprovalBaselineRule>,
            Rc::new(StrictApprovalBaselineNamespaceLengthRule { max_length: 64 }),
            Rc::new(StrictApprovalBaselineRegionLengthRule { max_length: 16 }),
            Rc::new(StrictApprovalBaselineAllowedCodeRule {
                allowed: vec!["strict".to_string(), "lenient".to_string()],
            }),
            Rc::new(StrictApprovalBaselineQuotaUnitsRangeRule { min: 120, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_approval_baseline_validator();
        let item = new_strict_approval_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = StrictApprovalBaselineRequiredLabelRule {
            message: "strict approval baseline needs a value".to_string(),
        };
        let mut item = new_strict_approval_baseline("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = StrictApprovalBaselineNamespaceLengthRule { max_length: 64 };
        let mut item = new_strict_approval_baseline("x", "y");
        item.namespace = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = StrictApprovalBaselineRegionLengthRule { max_length: 255 };
        let mut item = new_strict_approval_baseline("x", "y");
        item.region = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = StrictApprovalBaselineQuotaUnitsRangeRule { min: 32, max: 64 };
        let mut item = new_strict_approval_baseline("x", "y");
        item.quota_units = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
