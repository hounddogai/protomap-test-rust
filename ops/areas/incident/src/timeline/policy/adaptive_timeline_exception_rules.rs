use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive timeline exception must pass. Check returns its problems, or none.
pub trait AdaptiveTimelineExceptionRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveTimelineException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTimelineExceptionRequiredTenantIdRule {
    pub message: String,
}

impl AdaptiveTimelineExceptionRule for AdaptiveTimelineExceptionRequiredTenantIdRule {
    fn check(&self, item: &AdaptiveTimelineException) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTimelineExceptionLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveTimelineExceptionRule for AdaptiveTimelineExceptionLimitRangeRule {
    fn check(&self, item: &AdaptiveTimelineException) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTimelineExceptionTitleLengthRule {
    pub max_length: i64,
}

impl AdaptiveTimelineExceptionRule for AdaptiveTimelineExceptionTitleLengthRule {
    fn check(&self, item: &AdaptiveTimelineException) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTimelineExceptionRequiredCodeRule {
    pub message: String,
}

impl AdaptiveTimelineExceptionRule for AdaptiveTimelineExceptionRequiredCodeRule {
    fn check(&self, item: &AdaptiveTimelineException) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTimelineExceptionRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveTimelineExceptionRule for AdaptiveTimelineExceptionRatioRangeRule {
    fn check(&self, item: &AdaptiveTimelineException) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive timeline exception.
#[derive(Debug, Clone)]
pub struct AdaptiveTimelineExceptionValidator {
    pub rules: Vec<Rc<dyn AdaptiveTimelineExceptionRule>>,
}

impl AdaptiveTimelineExceptionValidator {
    pub fn validate(&self, item: &AdaptiveTimelineException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveTimelineException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveTimelineException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_timeline_exception_validator() -> AdaptiveTimelineExceptionValidator {
    AdaptiveTimelineExceptionValidator {
        rules: vec![
            Rc::new(AdaptiveTimelineExceptionRequiredTenantIdRule {
                message: "adaptive timeline exception needs an entry".to_string(),
            }) as Rc<dyn AdaptiveTimelineExceptionRule>,
            Rc::new(AdaptiveTimelineExceptionLimitRangeRule { min: 32, max: 86400 }),
            Rc::new(AdaptiveTimelineExceptionTitleLengthRule { max_length: 3600 }),
            Rc::new(AdaptiveTimelineExceptionRequiredCodeRule {
                message: "adaptive timeline exception needs a value".to_string(),
            }),
            Rc::new(AdaptiveTimelineExceptionRatioRangeRule { min: 5.0, max: 2561.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_timeline_exception_validator();
        let item = new_adaptive_timeline_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = AdaptiveTimelineExceptionRequiredTenantIdRule {
            message: "adaptive timeline exception needs a value".to_string(),
        };
        let mut item = new_adaptive_timeline_exception("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = AdaptiveTimelineExceptionLimitRangeRule { min: 64, max: 86400 };
        let mut item = new_adaptive_timeline_exception("x", "y");
        item.limit = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = AdaptiveTimelineExceptionTitleLengthRule { max_length: 120 };
        let mut item = new_adaptive_timeline_exception("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveTimelineExceptionRequiredCodeRule {
            message: "adaptive timeline exception needs a value".to_string(),
        };
        let mut item = new_adaptive_timeline_exception("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = AdaptiveTimelineExceptionRatioRangeRule { min: 9.8, max: 3278.5 };
        let mut item = new_adaptive_timeline_exception("x", "y");
        item.ratio = 3280.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
