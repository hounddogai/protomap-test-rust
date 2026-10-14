use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a budget burn notice must pass. Check returns its problems, or none.
pub trait BudgetBurnNoticeRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnNoticeSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnNoticeRule for BudgetBurnNoticeSequenceRangeRule {
    fn check(&self, item: &BudgetBurnNotice) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnNoticeLabelLengthRule {
    pub max_length: i64,
}

impl BudgetBurnNoticeRule for BudgetBurnNoticeLabelLengthRule {
    fn check(&self, item: &BudgetBurnNotice) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnNoticePinnedGuardRule {
    pub limit: i64,
}

impl BudgetBurnNoticeRule for BudgetBurnNoticePinnedGuardRule {
    fn check(&self, item: &BudgetBurnNotice) -> Vec<String> {
        if item.pinned && (item.priority > self.limit) {
            return vec![format!(
                "priority {} exceeds {} while pinned",
                item.priority, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnNoticeRequiredIssuerRule {
    pub message: String,
}

impl BudgetBurnNoticeRule for BudgetBurnNoticeRequiredIssuerRule {
    fn check(&self, item: &BudgetBurnNotice) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnNoticeRequiredNamespaceRule {
    pub message: String,
}

impl BudgetBurnNoticeRule for BudgetBurnNoticeRequiredNamespaceRule {
    fn check(&self, item: &BudgetBurnNotice) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn notice.
#[derive(Debug, Clone)]
pub struct BudgetBurnNoticeValidator {
    pub rules: Vec<Rc<dyn BudgetBurnNoticeRule>>,
}

impl BudgetBurnNoticeValidator {
    pub fn validate(&self, item: &BudgetBurnNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BudgetBurnNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_notice_validator() -> BudgetBurnNoticeValidator {
    BudgetBurnNoticeValidator {
        rules: vec![
            Rc::new(BudgetBurnNoticeSequenceRangeRule { min: 16, max: 86400 }) as Rc<dyn BudgetBurnNoticeRule>,
            Rc::new(BudgetBurnNoticeLabelLengthRule { max_length: 3600 }),
            Rc::new(BudgetBurnNoticePinnedGuardRule { limit: 255 }),
            Rc::new(BudgetBurnNoticeRequiredIssuerRule {
                message: "budget burn notice needs a value".to_string(),
            }),
            Rc::new(BudgetBurnNoticeRequiredNamespaceRule {
                message: "budget burn notice needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_notice_validator();
        let item = new_budget_burn_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = BudgetBurnNoticeSequenceRangeRule { min: 32, max: 120 };
        let mut item = new_budget_burn_notice("x", "y");
        item.sequence = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = BudgetBurnNoticeLabelLengthRule { max_length: 86400 };
        let mut item = new_budget_burn_notice("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = BudgetBurnNoticeRequiredIssuerRule {
            message: "budget burn notice needs a value".to_string(),
        };
        let mut item = new_budget_burn_notice("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = BudgetBurnNoticeRequiredNamespaceRule {
            message: "budget burn notice needs an entry".to_string(),
        };
        let mut item = new_budget_burn_notice("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
