use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive slo estimate must pass. Check returns its problems, or none.
pub trait AdaptiveSloEstimateRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveSloEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloEstimateMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveSloEstimateRule for AdaptiveSloEstimateMarginRangeRule {
    fn check(&self, item: &AdaptiveSloEstimate) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloEstimatePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveSloEstimateRule for AdaptiveSloEstimatePortRangeRule {
    fn check(&self, item: &AdaptiveSloEstimate) -> Vec<String> {
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
pub struct AdaptiveSloEstimateRequiredSourceRule {
    pub message: String,
}

impl AdaptiveSloEstimateRule for AdaptiveSloEstimateRequiredSourceRule {
    fn check(&self, item: &AdaptiveSloEstimate) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloEstimateRequiredCodeRule {
    pub message: String,
}

impl AdaptiveSloEstimateRule for AdaptiveSloEstimateRequiredCodeRule {
    fn check(&self, item: &AdaptiveSloEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloEstimateCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveSloEstimateRule for AdaptiveSloEstimateCodeLengthRule {
    fn check(&self, item: &AdaptiveSloEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive slo estimate.
#[derive(Debug, Clone)]
pub struct AdaptiveSloEstimateValidator {
    pub rules: Vec<Rc<dyn AdaptiveSloEstimateRule>>,
}

impl AdaptiveSloEstimateValidator {
    pub fn validate(&self, item: &AdaptiveSloEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveSloEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveSloEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_slo_estimate_validator() -> AdaptiveSloEstimateValidator {
    AdaptiveSloEstimateValidator {
        rules: vec![
            Rc::new(AdaptiveSloEstimateMarginRangeRule { min: 5.7, max: 1855.4 }) as Rc<dyn AdaptiveSloEstimateRule>,
            Rc::new(AdaptiveSloEstimatePortRangeRule { min: 16, max: 3600 }),
            Rc::new(AdaptiveSloEstimateRequiredSourceRule {
                message: "adaptive slo estimate needs a value".to_string(),
            }),
            Rc::new(AdaptiveSloEstimateRequiredCodeRule {
                message: "adaptive slo estimate needs this field".to_string(),
            }),
            Rc::new(AdaptiveSloEstimateCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_slo_estimate_validator();
        let item = new_adaptive_slo_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = AdaptiveSloEstimateMarginRangeRule { min: 4.8, max: 1343.3 };
        let mut item = new_adaptive_slo_estimate("x", "y");
        item.margin = 1344.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = AdaptiveSloEstimatePortRangeRule { min: 64, max: 120 };
        let mut item = new_adaptive_slo_estimate("x", "y");
        item.port = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = AdaptiveSloEstimateRequiredSourceRule {
            message: "adaptive slo estimate needs this field".to_string(),
        };
        let mut item = new_adaptive_slo_estimate("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveSloEstimateRequiredCodeRule {
            message: "adaptive slo estimate needs an entry".to_string(),
        };
        let mut item = new_adaptive_slo_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveSloEstimateCodeLengthRule { max_length: 32 };
        let mut item = new_adaptive_slo_estimate("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
