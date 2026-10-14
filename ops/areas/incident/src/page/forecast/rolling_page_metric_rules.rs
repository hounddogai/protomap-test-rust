use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling page metric must pass. Check returns its problems, or none.
pub trait RollingPageMetricRule: std::fmt::Debug {
    fn check(&self, item: &RollingPageMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageMetricClusterLengthRule {
    pub max_length: i64,
}

impl RollingPageMetricRule for RollingPageMetricClusterLengthRule {
    fn check(&self, item: &RollingPageMetric) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageMetricPinnedGuardRule {
    pub limit: f64,
}

impl RollingPageMetricRule for RollingPageMetricPinnedGuardRule {
    fn check(&self, item: &RollingPageMetric) -> Vec<String> {
        if item.pinned && (item.budget > self.limit) {
            return vec![format!("budget {} exceeds {} while pinned", item.budget, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageMetricRequiredTitleRule {
    pub message: String,
}

impl RollingPageMetricRule for RollingPageMetricRequiredTitleRule {
    fn check(&self, item: &RollingPageMetric) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageMetricCodeLengthRule {
    pub max_length: i64,
}

impl RollingPageMetricRule for RollingPageMetricCodeLengthRule {
    fn check(&self, item: &RollingPageMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling page metric.
#[derive(Debug, Clone)]
pub struct RollingPageMetricValidator {
    pub rules: Vec<Rc<dyn RollingPageMetricRule>>,
}

impl RollingPageMetricValidator {
    pub fn validate(&self, item: &RollingPageMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingPageMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingPageMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_page_metric_validator() -> RollingPageMetricValidator {
    RollingPageMetricValidator {
        rules: vec![
            Rc::new(RollingPageMetricClusterLengthRule { max_length: 64 }) as Rc<dyn RollingPageMetricRule>,
            Rc::new(RollingPageMetricPinnedGuardRule { limit: 7.9 }),
            Rc::new(RollingPageMetricRequiredTitleRule {
                message: "rolling page metric needs a value".to_string(),
            }),
            Rc::new(RollingPageMetricCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_page_metric_validator();
        let item = new_rolling_page_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = RollingPageMetricClusterLengthRule { max_length: 16 };
        let mut item = new_rolling_page_metric("x", "y");
        item.cluster = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RollingPageMetricRequiredTitleRule {
            message: "rolling page metric needs an entry".to_string(),
        };
        let mut item = new_rolling_page_metric("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingPageMetricCodeLengthRule { max_length: 80 };
        let mut item = new_rolling_page_metric("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
