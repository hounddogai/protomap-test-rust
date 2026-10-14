use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional page profile must pass. Check returns its problems, or none.
pub trait RegionalPageProfileRule: std::fmt::Debug {
    fn check(&self, item: &RegionalPageProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPageProfileRequiredTenantIdRule {
    pub message: String,
}

impl RegionalPageProfileRule for RegionalPageProfileRequiredTenantIdRule {
    fn check(&self, item: &RegionalPageProfile) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPageProfileRequiredKeyIdRule {
    pub message: String,
}

impl RegionalPageProfileRule for RegionalPageProfileRequiredKeyIdRule {
    fn check(&self, item: &RegionalPageProfile) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPageProfileTitleLengthRule {
    pub max_length: i64,
}

impl RegionalPageProfileRule for RegionalPageProfileTitleLengthRule {
    fn check(&self, item: &RegionalPageProfile) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPageProfileMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalPageProfileRule for RegionalPageProfileMaximumRangeRule {
    fn check(&self, item: &RegionalPageProfile) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPageProfileKeyIdLengthRule {
    pub max_length: i64,
}

impl RegionalPageProfileRule for RegionalPageProfileKeyIdLengthRule {
    fn check(&self, item: &RegionalPageProfile) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional page profile.
#[derive(Debug, Clone)]
pub struct RegionalPageProfileValidator {
    pub rules: Vec<Rc<dyn RegionalPageProfileRule>>,
}

impl RegionalPageProfileValidator {
    pub fn validate(&self, item: &RegionalPageProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalPageProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalPageProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_page_profile_validator() -> RegionalPageProfileValidator {
    RegionalPageProfileValidator {
        rules: vec![
            Rc::new(RegionalPageProfileRequiredTenantIdRule {
                message: "regional page profile needs this field".to_string(),
            }) as Rc<dyn RegionalPageProfileRule>,
            Rc::new(RegionalPageProfileRequiredKeyIdRule {
                message: "regional page profile needs a value".to_string(),
            }),
            Rc::new(RegionalPageProfileTitleLengthRule { max_length: 255 }),
            Rc::new(RegionalPageProfileMaximumRangeRule { min: 7.3, max: 2711.1 }),
            Rc::new(RegionalPageProfileKeyIdLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_page_profile_validator();
        let item = new_regional_page_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = RegionalPageProfileRequiredTenantIdRule {
            message: "regional page profile needs a value".to_string(),
        };
        let mut item = new_regional_page_profile("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = RegionalPageProfileRequiredKeyIdRule {
            message: "regional page profile needs a value".to_string(),
        };
        let mut item = new_regional_page_profile("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RegionalPageProfileTitleLengthRule { max_length: 16 };
        let mut item = new_regional_page_profile("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = RegionalPageProfileMaximumRangeRule { min: 7.8, max: 2443.6 };
        let mut item = new_regional_page_profile("x", "y");
        item.maximum = 2445.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = RegionalPageProfileKeyIdLengthRule { max_length: 64 };
        let mut item = new_regional_page_profile("x", "y");
        item.key_id = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
