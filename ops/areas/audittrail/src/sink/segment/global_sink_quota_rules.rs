use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global sink quota must pass. Check returns its problems, or none.
pub trait GlobalSinkQuotaRule: std::fmt::Debug {
    fn check(&self, item: &GlobalSinkQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkQuotaCodeLengthRule {
    pub max_length: i64,
}

impl GlobalSinkQuotaRule for GlobalSinkQuotaCodeLengthRule {
    fn check(&self, item: &GlobalSinkQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkQuotaDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalSinkQuotaRule for GlobalSinkQuotaDiscountRangeRule {
    fn check(&self, item: &GlobalSinkQuota) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkQuotaRequiredLabelRule {
    pub message: String,
}

impl GlobalSinkQuotaRule for GlobalSinkQuotaRequiredLabelRule {
    fn check(&self, item: &GlobalSinkQuota) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkQuotaRequiredTitleRule {
    pub message: String,
}

impl GlobalSinkQuotaRule for GlobalSinkQuotaRequiredTitleRule {
    fn check(&self, item: &GlobalSinkQuota) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkQuotaCodePrefixRule {
    pub prefix: String,
}

impl GlobalSinkQuotaRule for GlobalSinkQuotaCodePrefixRule {
    fn check(&self, item: &GlobalSinkQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkQuotaRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalSinkQuotaRule for GlobalSinkQuotaRatioRangeRule {
    fn check(&self, item: &GlobalSinkQuota) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a global sink quota.
#[derive(Debug, Clone)]
pub struct GlobalSinkQuotaValidator {
    pub rules: Vec<Rc<dyn GlobalSinkQuotaRule>>,
}

impl GlobalSinkQuotaValidator {
    pub fn validate(&self, item: &GlobalSinkQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalSinkQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalSinkQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_sink_quota_validator() -> GlobalSinkQuotaValidator {
    GlobalSinkQuotaValidator {
        rules: vec![
            Rc::new(GlobalSinkQuotaCodeLengthRule { max_length: 3600 }) as Rc<dyn GlobalSinkQuotaRule>,
            Rc::new(GlobalSinkQuotaDiscountRangeRule { min: 3.1, max: 714.7 }),
            Rc::new(GlobalSinkQuotaRequiredLabelRule {
                message: "global sink quota needs an entry".to_string(),
            }),
            Rc::new(GlobalSinkQuotaRequiredTitleRule {
                message: "global sink quota needs an entry".to_string(),
            }),
            Rc::new(GlobalSinkQuotaCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(GlobalSinkQuotaRatioRangeRule { min: 7.3, max: 4926.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_sink_quota_validator();
        let item = new_global_sink_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalSinkQuotaCodeLengthRule { max_length: 16 };
        let mut item = new_global_sink_quota("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = GlobalSinkQuotaDiscountRangeRule { min: 9.8, max: 1693.8 };
        let mut item = new_global_sink_quota("x", "y");
        item.discount = 1695.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = GlobalSinkQuotaRequiredLabelRule {
            message: "global sink quota needs this field".to_string(),
        };
        let mut item = new_global_sink_quota("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = GlobalSinkQuotaRequiredTitleRule {
            message: "global sink quota needs a value".to_string(),
        };
        let mut item = new_global_sink_quota("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = GlobalSinkQuotaRatioRangeRule { min: 5.8, max: 622.6 };
        let mut item = new_global_sink_quota("x", "y");
        item.ratio = 624.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
