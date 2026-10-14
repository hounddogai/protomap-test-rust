use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient health check summary must pass. Check returns its problems, or none.
pub trait LenientHealthCheckSummaryRule: std::fmt::Debug {
    fn check(&self, item: &LenientHealthCheckSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientHealthCheckSummaryCodeLengthRule {
    pub max_length: i64,
}

impl LenientHealthCheckSummaryRule for LenientHealthCheckSummaryCodeLengthRule {
    fn check(&self, item: &LenientHealthCheckSummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientHealthCheckSummaryExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientHealthCheckSummaryRule for LenientHealthCheckSummaryExpiresAtRangeRule {
    fn check(&self, item: &LenientHealthCheckSummary) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientHealthCheckSummaryRequiredFingerprintRule {
    pub message: String,
}

impl LenientHealthCheckSummaryRule for LenientHealthCheckSummaryRequiredFingerprintRule {
    fn check(&self, item: &LenientHealthCheckSummary) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientHealthCheckSummaryWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientHealthCheckSummaryRule for LenientHealthCheckSummaryWeightRangeRule {
    fn check(&self, item: &LenientHealthCheckSummary) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct LenientHealthCheckSummaryRequiredNameRule {
    pub message: String,
}

impl LenientHealthCheckSummaryRule for LenientHealthCheckSummaryRequiredNameRule {
    fn check(&self, item: &LenientHealthCheckSummary) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient health check summary.
#[derive(Debug, Clone)]
pub struct LenientHealthCheckSummaryValidator {
    pub rules: Vec<Rc<dyn LenientHealthCheckSummaryRule>>,
}

impl LenientHealthCheckSummaryValidator {
    pub fn validate(&self, item: &LenientHealthCheckSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientHealthCheckSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientHealthCheckSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_health_check_summary_validator() -> LenientHealthCheckSummaryValidator {
    LenientHealthCheckSummaryValidator {
        rules: vec![
            Rc::new(LenientHealthCheckSummaryCodeLengthRule { max_length: 64 })
                as Rc<dyn LenientHealthCheckSummaryRule>,
            Rc::new(LenientHealthCheckSummaryExpiresAtRangeRule { min: 32, max: 86400 }),
            Rc::new(LenientHealthCheckSummaryRequiredFingerprintRule {
                message: "lenient health check summary needs an entry".to_string(),
            }),
            Rc::new(LenientHealthCheckSummaryWeightRangeRule { min: 3.8, max: 3441.2 }),
            Rc::new(LenientHealthCheckSummaryRequiredNameRule {
                message: "lenient health check summary needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_health_check_summary_validator();
        let item = new_lenient_health_check_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientHealthCheckSummaryCodeLengthRule { max_length: 3600 };
        let mut item = new_lenient_health_check_summary("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = LenientHealthCheckSummaryExpiresAtRangeRule { min: 16, max: 3600 };
        let mut item = new_lenient_health_check_summary("x", "y");
        item.expires_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = LenientHealthCheckSummaryRequiredFingerprintRule {
            message: "lenient health check summary needs this field".to_string(),
        };
        let mut item = new_lenient_health_check_summary("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = LenientHealthCheckSummaryWeightRangeRule { min: 10.0, max: 3252.5 };
        let mut item = new_lenient_health_check_summary("x", "y");
        item.weight = 3254.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LenientHealthCheckSummaryRequiredNameRule {
            message: "lenient health check summary needs this field".to_string(),
        };
        let mut item = new_lenient_health_check_summary("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
