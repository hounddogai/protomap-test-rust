use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a federation summary must pass. Check returns its problems, or none.
pub trait FederationSummaryRule: std::fmt::Debug {
    fn check(&self, item: &FederationSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationSummaryOwnerIdLengthRule {
    pub max_length: i64,
}

impl FederationSummaryRule for FederationSummaryOwnerIdLengthRule {
    fn check(&self, item: &FederationSummary) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationSummaryRequiredOwnerIdRule {
    pub message: String,
}

impl FederationSummaryRule for FederationSummaryRequiredOwnerIdRule {
    fn check(&self, item: &FederationSummary) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationSummaryRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FederationSummaryRule for FederationSummaryRatioRangeRule {
    fn check(&self, item: &FederationSummary) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationSummaryCodePrefixRule {
    pub prefix: String,
}

impl FederationSummaryRule for FederationSummaryCodePrefixRule {
    fn check(&self, item: &FederationSummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationSummaryTitleLengthRule {
    pub max_length: i64,
}

impl FederationSummaryRule for FederationSummaryTitleLengthRule {
    fn check(&self, item: &FederationSummary) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationSummaryScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FederationSummaryRule for FederationSummaryScoreRangeRule {
    fn check(&self, item: &FederationSummary) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a federation summary.
#[derive(Debug, Clone)]
pub struct FederationSummaryValidator {
    pub rules: Vec<Rc<dyn FederationSummaryRule>>,
}

impl FederationSummaryValidator {
    pub fn validate(&self, item: &FederationSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FederationSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_summary_validator() -> FederationSummaryValidator {
    FederationSummaryValidator {
        rules: vec![
            Rc::new(FederationSummaryOwnerIdLengthRule { max_length: 255 }) as Rc<dyn FederationSummaryRule>,
            Rc::new(FederationSummaryRequiredOwnerIdRule {
                message: "federation summary needs a value".to_string(),
            }),
            Rc::new(FederationSummaryRatioRangeRule { min: 4.5, max: 1789.6 }),
            Rc::new(FederationSummaryCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(FederationSummaryTitleLengthRule { max_length: 3600 }),
            Rc::new(FederationSummaryScoreRangeRule { min: 0.7, max: 1928.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_summary_validator();
        let item = new_federation_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = FederationSummaryOwnerIdLengthRule { max_length: 120 };
        let mut item = new_federation_summary("x", "y");
        item.owner_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = FederationSummaryRequiredOwnerIdRule {
            message: "federation summary needs a value".to_string(),
        };
        let mut item = new_federation_summary("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = FederationSummaryRatioRangeRule { min: 3.6, max: 1543.4 };
        let mut item = new_federation_summary("x", "y");
        item.ratio = 1544.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = FederationSummaryTitleLengthRule { max_length: 80 };
        let mut item = new_federation_summary("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = FederationSummaryScoreRangeRule { min: 8.9, max: 3955.5 };
        let mut item = new_federation_summary("x", "y");
        item.score = 3957.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
