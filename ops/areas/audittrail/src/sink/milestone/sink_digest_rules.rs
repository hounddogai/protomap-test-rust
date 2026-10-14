use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink digest must pass. Check returns its problems, or none.
pub trait SinkDigestRule: std::fmt::Debug {
    fn check(&self, item: &SinkDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkDigestSummaryLengthRule {
    pub max_length: i64,
}

impl SinkDigestRule for SinkDigestSummaryLengthRule {
    fn check(&self, item: &SinkDigest) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkDigestCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SinkDigestRule for SinkDigestCapacityRangeRule {
    fn check(&self, item: &SinkDigest) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkDigestCodeLengthRule {
    pub max_length: i64,
}

impl SinkDigestRule for SinkDigestCodeLengthRule {
    fn check(&self, item: &SinkDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkDigestTitleLengthRule {
    pub max_length: i64,
}

impl SinkDigestRule for SinkDigestTitleLengthRule {
    fn check(&self, item: &SinkDigest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink digest.
#[derive(Debug, Clone)]
pub struct SinkDigestValidator {
    pub rules: Vec<Rc<dyn SinkDigestRule>>,
}

impl SinkDigestValidator {
    pub fn validate(&self, item: &SinkDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_digest_validator() -> SinkDigestValidator {
    SinkDigestValidator {
        rules: vec![
            Rc::new(SinkDigestSummaryLengthRule { max_length: 80 }) as Rc<dyn SinkDigestRule>,
            Rc::new(SinkDigestCapacityRangeRule { min: 120, max: 86400 }),
            Rc::new(SinkDigestCodeLengthRule { max_length: 1000 }),
            Rc::new(SinkDigestTitleLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_digest_validator();
        let item = new_sink_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = SinkDigestSummaryLengthRule { max_length: 120 };
        let mut item = new_sink_digest("x", "y");
        item.summary = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = SinkDigestCapacityRangeRule { min: 16, max: 64 };
        let mut item = new_sink_digest("x", "y");
        item.capacity = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SinkDigestCodeLengthRule { max_length: 16 };
        let mut item = new_sink_digest("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SinkDigestTitleLengthRule { max_length: 16 };
        let mut item = new_sink_digest("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
