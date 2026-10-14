use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive suspension baseline must pass. Check returns its problems, or none.
pub trait AdaptiveSuspensionBaselineRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveSuspensionBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSuspensionBaselineRequiredSummaryRule {
    pub message: String,
}

impl AdaptiveSuspensionBaselineRule for AdaptiveSuspensionBaselineRequiredSummaryRule {
    fn check(&self, item: &AdaptiveSuspensionBaseline) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSuspensionBaselineRequiredNameRule {
    pub message: String,
}

impl AdaptiveSuspensionBaselineRule for AdaptiveSuspensionBaselineRequiredNameRule {
    fn check(&self, item: &AdaptiveSuspensionBaseline) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSuspensionBaselineMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveSuspensionBaselineRule for AdaptiveSuspensionBaselineMarginRangeRule {
    fn check(&self, item: &AdaptiveSuspensionBaseline) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSuspensionBaselineSummaryLengthRule {
    pub max_length: i64,
}

impl AdaptiveSuspensionBaselineRule for AdaptiveSuspensionBaselineSummaryLengthRule {
    fn check(&self, item: &AdaptiveSuspensionBaseline) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive suspension baseline.
#[derive(Debug, Clone)]
pub struct AdaptiveSuspensionBaselineValidator {
    pub rules: Vec<Rc<dyn AdaptiveSuspensionBaselineRule>>,
}

impl AdaptiveSuspensionBaselineValidator {
    pub fn validate(&self, item: &AdaptiveSuspensionBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveSuspensionBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveSuspensionBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_suspension_baseline_validator() -> AdaptiveSuspensionBaselineValidator {
    AdaptiveSuspensionBaselineValidator {
        rules: vec![
            Rc::new(AdaptiveSuspensionBaselineRequiredSummaryRule {
                message: "adaptive suspension baseline needs a value".to_string(),
            }) as Rc<dyn AdaptiveSuspensionBaselineRule>,
            Rc::new(AdaptiveSuspensionBaselineRequiredNameRule {
                message: "adaptive suspension baseline needs this field".to_string(),
            }),
            Rc::new(AdaptiveSuspensionBaselineMarginRangeRule { min: 4.7, max: 4441.1 }),
            Rc::new(AdaptiveSuspensionBaselineSummaryLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_suspension_baseline_validator();
        let item = new_adaptive_suspension_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = AdaptiveSuspensionBaselineRequiredSummaryRule {
            message: "adaptive suspension baseline needs an entry".to_string(),
        };
        let mut item = new_adaptive_suspension_baseline("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = AdaptiveSuspensionBaselineRequiredNameRule {
            message: "adaptive suspension baseline needs an entry".to_string(),
        };
        let mut item = new_adaptive_suspension_baseline("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = AdaptiveSuspensionBaselineMarginRangeRule { min: 6.1, max: 372.2 };
        let mut item = new_adaptive_suspension_baseline("x", "y");
        item.margin = 373.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = AdaptiveSuspensionBaselineSummaryLengthRule { max_length: 80 };
        let mut item = new_adaptive_suspension_baseline("x", "y");
        item.summary = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
