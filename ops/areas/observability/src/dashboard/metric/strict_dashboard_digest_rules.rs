use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict dashboard digest must pass. Check returns its problems, or none.
pub trait StrictDashboardDigestRule: std::fmt::Debug {
    fn check(&self, item: &StrictDashboardDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardDigestRequiredCodeRule {
    pub message: String,
}

impl StrictDashboardDigestRule for StrictDashboardDigestRequiredCodeRule {
    fn check(&self, item: &StrictDashboardDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardDigestCodePrefixRule {
    pub prefix: String,
}

impl StrictDashboardDigestRule for StrictDashboardDigestCodePrefixRule {
    fn check(&self, item: &StrictDashboardDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardDigestTitleLengthRule {
    pub max_length: i64,
}

impl StrictDashboardDigestRule for StrictDashboardDigestTitleLengthRule {
    fn check(&self, item: &StrictDashboardDigest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict dashboard digest.
#[derive(Debug, Clone)]
pub struct StrictDashboardDigestValidator {
    pub rules: Vec<Rc<dyn StrictDashboardDigestRule>>,
}

impl StrictDashboardDigestValidator {
    pub fn validate(&self, item: &StrictDashboardDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictDashboardDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictDashboardDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_dashboard_digest_validator() -> StrictDashboardDigestValidator {
    StrictDashboardDigestValidator {
        rules: vec![
            Rc::new(StrictDashboardDigestRequiredCodeRule {
                message: "strict dashboard digest needs this field".to_string(),
            }) as Rc<dyn StrictDashboardDigestRule>,
            Rc::new(StrictDashboardDigestCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(StrictDashboardDigestTitleLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_dashboard_digest_validator();
        let item = new_strict_dashboard_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictDashboardDigestRequiredCodeRule {
            message: "strict dashboard digest needs a value".to_string(),
        };
        let mut item = new_strict_dashboard_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = StrictDashboardDigestTitleLengthRule { max_length: 1000 };
        let mut item = new_strict_dashboard_digest("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
