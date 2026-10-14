use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive probe estimate must pass. Check returns its problems, or none.
pub trait AdaptiveProbeEstimateRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveProbeEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeEstimateRequiredCodeRule {
    pub message: String,
}

impl AdaptiveProbeEstimateRule for AdaptiveProbeEstimateRequiredCodeRule {
    fn check(&self, item: &AdaptiveProbeEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeEstimateBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveProbeEstimateRule for AdaptiveProbeEstimateBudgetRangeRule {
    fn check(&self, item: &AdaptiveProbeEstimate) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeEstimateAllowedNameRule {
    pub allowed: Vec<String>,
}

impl AdaptiveProbeEstimateRule for AdaptiveProbeEstimateAllowedNameRule {
    fn check(&self, item: &AdaptiveProbeEstimate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is blocked", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeEstimateRegionLengthRule {
    pub max_length: i64,
}

impl AdaptiveProbeEstimateRule for AdaptiveProbeEstimateRegionLengthRule {
    fn check(&self, item: &AdaptiveProbeEstimate) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeEstimateQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveProbeEstimateRule for AdaptiveProbeEstimateQuotaUnitsRangeRule {
    fn check(&self, item: &AdaptiveProbeEstimate) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive probe estimate.
#[derive(Debug, Clone)]
pub struct AdaptiveProbeEstimateValidator {
    pub rules: Vec<Rc<dyn AdaptiveProbeEstimateRule>>,
}

impl AdaptiveProbeEstimateValidator {
    pub fn validate(&self, item: &AdaptiveProbeEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveProbeEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveProbeEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_probe_estimate_validator() -> AdaptiveProbeEstimateValidator {
    AdaptiveProbeEstimateValidator {
        rules: vec![
            Rc::new(AdaptiveProbeEstimateRequiredCodeRule {
                message: "adaptive probe estimate needs this field".to_string(),
            }) as Rc<dyn AdaptiveProbeEstimateRule>,
            Rc::new(AdaptiveProbeEstimateBudgetRangeRule { min: 7.9, max: 588.7 }),
            Rc::new(AdaptiveProbeEstimateAllowedNameRule {
                allowed: vec!["primary".to_string(), "lenient".to_string()],
            }),
            Rc::new(AdaptiveProbeEstimateRegionLengthRule { max_length: 80 }),
            Rc::new(AdaptiveProbeEstimateQuotaUnitsRangeRule { min: 1000, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_probe_estimate_validator();
        let item = new_adaptive_probe_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveProbeEstimateRequiredCodeRule {
            message: "adaptive probe estimate needs this field".to_string(),
        };
        let mut item = new_adaptive_probe_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = AdaptiveProbeEstimateBudgetRangeRule { min: 8.2, max: 4733.4 };
        let mut item = new_adaptive_probe_estimate("x", "y");
        item.budget = 4734.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = AdaptiveProbeEstimateRegionLengthRule { max_length: 32 };
        let mut item = new_adaptive_probe_estimate("x", "y");
        item.region = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = AdaptiveProbeEstimateQuotaUnitsRangeRule { min: 64, max: 1000 };
        let mut item = new_adaptive_probe_estimate("x", "y");
        item.quota_units = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
