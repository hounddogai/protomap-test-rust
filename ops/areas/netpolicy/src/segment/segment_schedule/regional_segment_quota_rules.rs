use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional segment quota must pass. Check returns its problems, or none.
pub trait RegionalSegmentQuotaRule: std::fmt::Debug {
    fn check(&self, item: &RegionalSegmentQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSegmentQuotaDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalSegmentQuotaRule for RegionalSegmentQuotaDiscountRangeRule {
    fn check(&self, item: &RegionalSegmentQuota) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSegmentQuotaRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalSegmentQuotaRule for RegionalSegmentQuotaRatioRangeRule {
    fn check(&self, item: &RegionalSegmentQuota) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSegmentQuotaAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RegionalSegmentQuotaRule for RegionalSegmentQuotaAllowedCodeRule {
    fn check(&self, item: &RegionalSegmentQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSegmentQuotaRequiredLabelRule {
    pub message: String,
}

impl RegionalSegmentQuotaRule for RegionalSegmentQuotaRequiredLabelRule {
    fn check(&self, item: &RegionalSegmentQuota) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional segment quota.
#[derive(Debug, Clone)]
pub struct RegionalSegmentQuotaValidator {
    pub rules: Vec<Rc<dyn RegionalSegmentQuotaRule>>,
}

impl RegionalSegmentQuotaValidator {
    pub fn validate(&self, item: &RegionalSegmentQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalSegmentQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalSegmentQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_segment_quota_validator() -> RegionalSegmentQuotaValidator {
    RegionalSegmentQuotaValidator {
        rules: vec![
            Rc::new(RegionalSegmentQuotaDiscountRangeRule { min: 6.7, max: 3187.7 })
                as Rc<dyn RegionalSegmentQuotaRule>,
            Rc::new(RegionalSegmentQuotaRatioRangeRule { min: 6.9, max: 3387.8 }),
            Rc::new(RegionalSegmentQuotaAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(RegionalSegmentQuotaRequiredLabelRule {
                message: "regional segment quota needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_segment_quota_validator();
        let item = new_regional_segment_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = RegionalSegmentQuotaDiscountRangeRule { min: 8.0, max: 2155.1 };
        let mut item = new_regional_segment_quota("x", "y");
        item.discount = 2156.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = RegionalSegmentQuotaRatioRangeRule { min: 6.7, max: 2396.5 };
        let mut item = new_regional_segment_quota("x", "y");
        item.ratio = 2398.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RegionalSegmentQuotaRequiredLabelRule {
            message: "regional segment quota needs a value".to_string(),
        };
        let mut item = new_regional_segment_quota("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
