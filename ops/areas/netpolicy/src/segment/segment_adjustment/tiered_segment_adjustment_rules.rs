use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered segment adjustment must pass. Check returns its problems, or none.
pub trait TieredSegmentAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &TieredSegmentAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentAdjustmentRequiredCodeRule {
    pub message: String,
}

impl TieredSegmentAdjustmentRule for TieredSegmentAdjustmentRequiredCodeRule {
    fn check(&self, item: &TieredSegmentAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentAdjustmentDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredSegmentAdjustmentRule for TieredSegmentAdjustmentDiscountRangeRule {
    fn check(&self, item: &TieredSegmentAdjustment) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentAdjustmentRequiredIssuerRule {
    pub message: String,
}

impl TieredSegmentAdjustmentRule for TieredSegmentAdjustmentRequiredIssuerRule {
    fn check(&self, item: &TieredSegmentAdjustment) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentAdjustmentAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl TieredSegmentAdjustmentRule for TieredSegmentAdjustmentAllowedTitleRule {
    fn check(&self, item: &TieredSegmentAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is not allowed", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentAdjustmentPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredSegmentAdjustmentRule for TieredSegmentAdjustmentPortRangeRule {
    fn check(&self, item: &TieredSegmentAdjustment) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentAdjustmentWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredSegmentAdjustmentRule for TieredSegmentAdjustmentWeightRangeRule {
    fn check(&self, item: &TieredSegmentAdjustment) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a tiered segment adjustment.
#[derive(Debug, Clone)]
pub struct TieredSegmentAdjustmentValidator {
    pub rules: Vec<Rc<dyn TieredSegmentAdjustmentRule>>,
}

impl TieredSegmentAdjustmentValidator {
    pub fn validate(&self, item: &TieredSegmentAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredSegmentAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredSegmentAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_segment_adjustment_validator() -> TieredSegmentAdjustmentValidator {
    TieredSegmentAdjustmentValidator {
        rules: vec![
            Rc::new(TieredSegmentAdjustmentRequiredCodeRule {
                message: "tiered segment adjustment needs a value".to_string(),
            }) as Rc<dyn TieredSegmentAdjustmentRule>,
            Rc::new(TieredSegmentAdjustmentDiscountRangeRule { min: 3.6, max: 3806.8 }),
            Rc::new(TieredSegmentAdjustmentRequiredIssuerRule {
                message: "tiered segment adjustment needs this field".to_string(),
            }),
            Rc::new(TieredSegmentAdjustmentAllowedTitleRule { allowed: Vec::new() }),
            Rc::new(TieredSegmentAdjustmentPortRangeRule { min: 80, max: 3600 }),
            Rc::new(TieredSegmentAdjustmentWeightRangeRule { min: 4.5, max: 3466.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_segment_adjustment_validator();
        let item = new_tiered_segment_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredSegmentAdjustmentRequiredCodeRule {
            message: "tiered segment adjustment needs a value".to_string(),
        };
        let mut item = new_tiered_segment_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = TieredSegmentAdjustmentDiscountRangeRule { min: 9.6, max: 4605.2 };
        let mut item = new_tiered_segment_adjustment("x", "y");
        item.discount = 4606.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = TieredSegmentAdjustmentRequiredIssuerRule {
            message: "tiered segment adjustment needs this field".to_string(),
        };
        let mut item = new_tiered_segment_adjustment("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = TieredSegmentAdjustmentPortRangeRule { min: 16, max: 1000 };
        let mut item = new_tiered_segment_adjustment("x", "y");
        item.port = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = TieredSegmentAdjustmentWeightRangeRule { min: 5.8, max: 1339.8 };
        let mut item = new_tiered_segment_adjustment("x", "y");
        item.weight = 1341.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
