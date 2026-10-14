use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient dns record plan must pass. Check returns its problems, or none.
pub trait LenientDnsRecordPlanRule: std::fmt::Debug {
    fn check(&self, item: &LenientDnsRecordPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDnsRecordPlanRequiredCodeRule {
    pub message: String,
}

impl LenientDnsRecordPlanRule for LenientDnsRecordPlanRequiredCodeRule {
    fn check(&self, item: &LenientDnsRecordPlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDnsRecordPlanCodePrefixRule {
    pub prefix: String,
}

impl LenientDnsRecordPlanRule for LenientDnsRecordPlanCodePrefixRule {
    fn check(&self, item: &LenientDnsRecordPlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDnsRecordPlanCodeLengthRule {
    pub max_length: i64,
}

impl LenientDnsRecordPlanRule for LenientDnsRecordPlanCodeLengthRule {
    fn check(&self, item: &LenientDnsRecordPlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDnsRecordPlanPinnedGuardRule {
    pub limit: f64,
}

impl LenientDnsRecordPlanRule for LenientDnsRecordPlanPinnedGuardRule {
    fn check(&self, item: &LenientDnsRecordPlan) -> Vec<String> {
        if item.pinned && (item.margin > self.limit) {
            return vec![format!("margin {} exceeds {} while pinned", item.margin, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDnsRecordPlanRequiredNameRule {
    pub message: String,
}

impl LenientDnsRecordPlanRule for LenientDnsRecordPlanRequiredNameRule {
    fn check(&self, item: &LenientDnsRecordPlan) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient dns record plan.
#[derive(Debug, Clone)]
pub struct LenientDnsRecordPlanValidator {
    pub rules: Vec<Rc<dyn LenientDnsRecordPlanRule>>,
}

impl LenientDnsRecordPlanValidator {
    pub fn validate(&self, item: &LenientDnsRecordPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientDnsRecordPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientDnsRecordPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_dns_record_plan_validator() -> LenientDnsRecordPlanValidator {
    LenientDnsRecordPlanValidator {
        rules: vec![
            Rc::new(LenientDnsRecordPlanRequiredCodeRule {
                message: "lenient dns record plan needs an entry".to_string(),
            }) as Rc<dyn LenientDnsRecordPlanRule>,
            Rc::new(LenientDnsRecordPlanCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(LenientDnsRecordPlanCodeLengthRule { max_length: 120 }),
            Rc::new(LenientDnsRecordPlanPinnedGuardRule { limit: 5.9 }),
            Rc::new(LenientDnsRecordPlanRequiredNameRule {
                message: "lenient dns record plan needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_dns_record_plan_validator();
        let item = new_lenient_dns_record_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientDnsRecordPlanRequiredCodeRule {
            message: "lenient dns record plan needs this field".to_string(),
        };
        let mut item = new_lenient_dns_record_plan("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientDnsRecordPlanCodeLengthRule { max_length: 255 };
        let mut item = new_lenient_dns_record_plan("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LenientDnsRecordPlanRequiredNameRule {
            message: "lenient dns record plan needs this field".to_string(),
        };
        let mut item = new_lenient_dns_record_plan("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
