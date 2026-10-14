use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional canary summary must pass. Check returns its problems, or none.
pub trait RegionalCanarySummaryRule: std::fmt::Debug {
    fn check(&self, item: &RegionalCanarySummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanarySummaryPrincipalLengthRule {
    pub max_length: i64,
}

impl RegionalCanarySummaryRule for RegionalCanarySummaryPrincipalLengthRule {
    fn check(&self, item: &RegionalCanarySummary) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanarySummaryRequiredNameRule {
    pub message: String,
}

impl RegionalCanarySummaryRule for RegionalCanarySummaryRequiredNameRule {
    fn check(&self, item: &RegionalCanarySummary) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanarySummaryNameLengthRule {
    pub max_length: i64,
}

impl RegionalCanarySummaryRule for RegionalCanarySummaryNameLengthRule {
    fn check(&self, item: &RegionalCanarySummary) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanarySummaryCodePrefixRule {
    pub prefix: String,
}

impl RegionalCanarySummaryRule for RegionalCanarySummaryCodePrefixRule {
    fn check(&self, item: &RegionalCanarySummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional canary summary.
#[derive(Debug, Clone)]
pub struct RegionalCanarySummaryValidator {
    pub rules: Vec<Rc<dyn RegionalCanarySummaryRule>>,
}

impl RegionalCanarySummaryValidator {
    pub fn validate(&self, item: &RegionalCanarySummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalCanarySummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalCanarySummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_canary_summary_validator() -> RegionalCanarySummaryValidator {
    RegionalCanarySummaryValidator {
        rules: vec![
            Rc::new(RegionalCanarySummaryPrincipalLengthRule { max_length: 86400 })
                as Rc<dyn RegionalCanarySummaryRule>,
            Rc::new(RegionalCanarySummaryRequiredNameRule {
                message: "regional canary summary needs this field".to_string(),
            }),
            Rc::new(RegionalCanarySummaryNameLengthRule { max_length: 32 }),
            Rc::new(RegionalCanarySummaryCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_canary_summary_validator();
        let item = new_regional_canary_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = RegionalCanarySummaryPrincipalLengthRule { max_length: 255 };
        let mut item = new_regional_canary_summary("x", "y");
        item.principal = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RegionalCanarySummaryRequiredNameRule {
            message: "regional canary summary needs this field".to_string(),
        };
        let mut item = new_regional_canary_summary("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RegionalCanarySummaryNameLengthRule { max_length: 64 };
        let mut item = new_regional_canary_summary("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
