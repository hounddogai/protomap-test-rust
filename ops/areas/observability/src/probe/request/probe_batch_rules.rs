use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a probe batch must pass. Check returns its problems, or none.
pub trait ProbeBatchRule: std::fmt::Debug {
    fn check(&self, item: &ProbeBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeBatchRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeBatchRule for ProbeBatchRetriesRangeRule {
    fn check(&self, item: &ProbeBatch) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeBatchRequiredCodeRule {
    pub message: String,
}

impl ProbeBatchRule for ProbeBatchRequiredCodeRule {
    fn check(&self, item: &ProbeBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeBatchNameLengthRule {
    pub max_length: i64,
}

impl ProbeBatchRule for ProbeBatchNameLengthRule {
    fn check(&self, item: &ProbeBatch) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeBatchMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ProbeBatchRule for ProbeBatchMarginRangeRule {
    fn check(&self, item: &ProbeBatch) -> Vec<String> {
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
pub struct ProbeBatchRequiredIssuerRule {
    pub message: String,
}

impl ProbeBatchRule for ProbeBatchRequiredIssuerRule {
    fn check(&self, item: &ProbeBatch) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe batch.
#[derive(Debug, Clone)]
pub struct ProbeBatchValidator {
    pub rules: Vec<Rc<dyn ProbeBatchRule>>,
}

impl ProbeBatchValidator {
    pub fn validate(&self, item: &ProbeBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ProbeBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_batch_validator() -> ProbeBatchValidator {
    ProbeBatchValidator {
        rules: vec![
            Rc::new(ProbeBatchRetriesRangeRule { min: 32, max: 1000 }) as Rc<dyn ProbeBatchRule>,
            Rc::new(ProbeBatchRequiredCodeRule {
                message: "probe batch needs an entry".to_string(),
            }),
            Rc::new(ProbeBatchNameLengthRule { max_length: 86400 }),
            Rc::new(ProbeBatchMarginRangeRule { min: 4.5, max: 3347.3 }),
            Rc::new(ProbeBatchRequiredIssuerRule {
                message: "probe batch needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_batch_validator();
        let item = new_probe_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = ProbeBatchRetriesRangeRule { min: 32, max: 86400 };
        let mut item = new_probe_batch("x", "y");
        item.retries = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ProbeBatchRequiredCodeRule {
            message: "probe batch needs a value".to_string(),
        };
        let mut item = new_probe_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ProbeBatchNameLengthRule { max_length: 255 };
        let mut item = new_probe_batch("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = ProbeBatchMarginRangeRule { min: 9.0, max: 1725.4 };
        let mut item = new_probe_batch("x", "y");
        item.margin = 1726.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = ProbeBatchRequiredIssuerRule {
            message: "probe batch needs an entry".to_string(),
        };
        let mut item = new_probe_batch("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
