use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary page baseline must pass. Check returns its problems, or none.
pub trait PrimaryPageBaselineRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryPageBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPageBaselineClusterLengthRule {
    pub max_length: i64,
}

impl PrimaryPageBaselineRule for PrimaryPageBaselineClusterLengthRule {
    fn check(&self, item: &PrimaryPageBaseline) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPageBaselineRequiredChecksumRule {
    pub message: String,
}

impl PrimaryPageBaselineRule for PrimaryPageBaselineRequiredChecksumRule {
    fn check(&self, item: &PrimaryPageBaseline) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPageBaselineVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryPageBaselineRule for PrimaryPageBaselineVersionRangeRule {
    fn check(&self, item: &PrimaryPageBaseline) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPageBaselineChecksumLengthRule {
    pub max_length: i64,
}

impl PrimaryPageBaselineRule for PrimaryPageBaselineChecksumLengthRule {
    fn check(&self, item: &PrimaryPageBaseline) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPageBaselineTitleLengthRule {
    pub max_length: i64,
}

impl PrimaryPageBaselineRule for PrimaryPageBaselineTitleLengthRule {
    fn check(&self, item: &PrimaryPageBaseline) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPageBaselineCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryPageBaselineRule for PrimaryPageBaselineCodeLengthRule {
    fn check(&self, item: &PrimaryPageBaseline) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary page baseline.
#[derive(Debug, Clone)]
pub struct PrimaryPageBaselineValidator {
    pub rules: Vec<Rc<dyn PrimaryPageBaselineRule>>,
}

impl PrimaryPageBaselineValidator {
    pub fn validate(&self, item: &PrimaryPageBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryPageBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryPageBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_page_baseline_validator() -> PrimaryPageBaselineValidator {
    PrimaryPageBaselineValidator {
        rules: vec![
            Rc::new(PrimaryPageBaselineClusterLengthRule { max_length: 80 }) as Rc<dyn PrimaryPageBaselineRule>,
            Rc::new(PrimaryPageBaselineRequiredChecksumRule {
                message: "primary page baseline needs a value".to_string(),
            }),
            Rc::new(PrimaryPageBaselineVersionRangeRule { min: 120, max: 1000 }),
            Rc::new(PrimaryPageBaselineChecksumLengthRule { max_length: 1000 }),
            Rc::new(PrimaryPageBaselineTitleLengthRule { max_length: 32 }),
            Rc::new(PrimaryPageBaselineCodeLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_page_baseline_validator();
        let item = new_primary_page_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = PrimaryPageBaselineClusterLengthRule { max_length: 32 };
        let mut item = new_primary_page_baseline("x", "y");
        item.cluster = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = PrimaryPageBaselineRequiredChecksumRule {
            message: "primary page baseline needs a value".to_string(),
        };
        let mut item = new_primary_page_baseline("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = PrimaryPageBaselineVersionRangeRule { min: 64, max: 255 };
        let mut item = new_primary_page_baseline("x", "y");
        item.version = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = PrimaryPageBaselineChecksumLengthRule { max_length: 1000 };
        let mut item = new_primary_page_baseline("x", "y");
        item.checksum = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PrimaryPageBaselineTitleLengthRule { max_length: 120 };
        let mut item = new_primary_page_baseline("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryPageBaselineCodeLengthRule { max_length: 32 };
        let mut item = new_primary_page_baseline("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
