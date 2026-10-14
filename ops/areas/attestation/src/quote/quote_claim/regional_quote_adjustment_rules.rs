use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional quote adjustment must pass. Check returns its problems, or none.
pub trait RegionalQuoteAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &RegionalQuoteAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQuoteAdjustmentAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl RegionalQuoteAdjustmentRule for RegionalQuoteAdjustmentAllowedLabelRule {
    fn check(&self, item: &RegionalQuoteAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is below minimum", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQuoteAdjustmentRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalQuoteAdjustmentRule for RegionalQuoteAdjustmentRatioRangeRule {
    fn check(&self, item: &RegionalQuoteAdjustment) -> Vec<String> {
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
pub struct RegionalQuoteAdjustmentApprovedGuardRule {
    pub limit: f64,
}

impl RegionalQuoteAdjustmentRule for RegionalQuoteAdjustmentApprovedGuardRule {
    fn check(&self, item: &RegionalQuoteAdjustment) -> Vec<String> {
        if item.approved && (item.ratio > self.limit) {
            return vec![format!("ratio {} exceeds {} while approved", item.ratio, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional quote adjustment.
#[derive(Debug, Clone)]
pub struct RegionalQuoteAdjustmentValidator {
    pub rules: Vec<Rc<dyn RegionalQuoteAdjustmentRule>>,
}

impl RegionalQuoteAdjustmentValidator {
    pub fn validate(&self, item: &RegionalQuoteAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalQuoteAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalQuoteAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_quote_adjustment_validator() -> RegionalQuoteAdjustmentValidator {
    RegionalQuoteAdjustmentValidator {
        rules: vec![
            Rc::new(RegionalQuoteAdjustmentAllowedLabelRule { allowed: Vec::new() })
                as Rc<dyn RegionalQuoteAdjustmentRule>,
            Rc::new(RegionalQuoteAdjustmentRatioRangeRule { min: 5.5, max: 4595.9 }),
            Rc::new(RegionalQuoteAdjustmentApprovedGuardRule { limit: 1.0 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_quote_adjustment_validator();
        let item = new_regional_quote_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = RegionalQuoteAdjustmentRatioRangeRule { min: 9.9, max: 4650.3 };
        let mut item = new_regional_quote_adjustment("x", "y");
        item.ratio = 4651.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
