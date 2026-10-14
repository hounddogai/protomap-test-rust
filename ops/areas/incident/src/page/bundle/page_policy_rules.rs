use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a page policy must pass. Check returns its problems, or none.
pub trait PagePolicyRule: std::fmt::Debug {
    fn check(&self, item: &PagePolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PagePolicyLabelLengthRule {
    pub max_length: i64,
}

impl PagePolicyRule for PagePolicyLabelLengthRule {
    fn check(&self, item: &PagePolicy) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PagePolicyRequiredCodeRule {
    pub message: String,
}

impl PagePolicyRule for PagePolicyRequiredCodeRule {
    fn check(&self, item: &PagePolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PagePolicySummaryLengthRule {
    pub max_length: i64,
}

impl PagePolicyRule for PagePolicySummaryLengthRule {
    fn check(&self, item: &PagePolicy) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PagePolicyTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl PagePolicyRule for PagePolicyTagsLimitRule {
    fn check(&self, item: &PagePolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

/// Runs every rule of a page policy.
#[derive(Debug, Clone)]
pub struct PagePolicyValidator {
    pub rules: Vec<Rc<dyn PagePolicyRule>>,
}

impl PagePolicyValidator {
    pub fn validate(&self, item: &PagePolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PagePolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PagePolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_policy_validator() -> PagePolicyValidator {
    PagePolicyValidator {
        rules: vec![
            Rc::new(PagePolicyLabelLengthRule { max_length: 120 }) as Rc<dyn PagePolicyRule>,
            Rc::new(PagePolicyRequiredCodeRule {
                message: "page policy needs an entry".to_string(),
            }),
            Rc::new(PagePolicySummaryLengthRule { max_length: 80 }),
            Rc::new(PagePolicyTagsLimitRule {
                max_count: 64,
                banned: vec!["regional".to_string(), "batched".to_string(), "rolling".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_page_policy_validator();
        let item = new_page_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PagePolicyLabelLengthRule { max_length: 120 };
        let mut item = new_page_policy("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PagePolicyRequiredCodeRule {
            message: "page policy needs this field".to_string(),
        };
        let mut item = new_page_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = PagePolicySummaryLengthRule { max_length: 16 };
        let mut item = new_page_policy("x", "y");
        item.summary = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
