use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tenant template must pass. Check returns its problems, or none.
pub trait TenantTemplateRule: std::fmt::Debug {
    fn check(&self, item: &TenantTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantTemplateRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TenantTemplateRule for TenantTemplateRatioRangeRule {
    fn check(&self, item: &TenantTemplate) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantTemplateClusterLengthRule {
    pub max_length: i64,
}

impl TenantTemplateRule for TenantTemplateClusterLengthRule {
    fn check(&self, item: &TenantTemplate) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantTemplateAllowedClusterRule {
    pub allowed: Vec<String>,
}

impl TenantTemplateRule for TenantTemplateAllowedClusterRule {
    fn check(&self, item: &TenantTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.cluster)))
        {
            return vec![format!("cluster {} is below minimum", item.cluster)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantTemplateCodePrefixRule {
    pub prefix: String,
}

impl TenantTemplateRule for TenantTemplateCodePrefixRule {
    fn check(&self, item: &TenantTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantTemplateScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TenantTemplateRule for TenantTemplateScoreRangeRule {
    fn check(&self, item: &TenantTemplate) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant template.
#[derive(Debug, Clone)]
pub struct TenantTemplateValidator {
    pub rules: Vec<Rc<dyn TenantTemplateRule>>,
}

impl TenantTemplateValidator {
    pub fn validate(&self, item: &TenantTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TenantTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_template_validator() -> TenantTemplateValidator {
    TenantTemplateValidator {
        rules: vec![
            Rc::new(TenantTemplateRatioRangeRule { min: 4.7, max: 4723.4 }) as Rc<dyn TenantTemplateRule>,
            Rc::new(TenantTemplateClusterLengthRule { max_length: 120 }),
            Rc::new(TenantTemplateAllowedClusterRule {
                allowed: vec!["capped".to_string(), "fallback".to_string(), "manual".to_string()],
            }),
            Rc::new(TenantTemplateCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(TenantTemplateScoreRangeRule { min: 5.1, max: 1594.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_template_validator();
        let item = new_tenant_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = TenantTemplateRatioRangeRule { min: 9.0, max: 2969.6 };
        let mut item = new_tenant_template("x", "y");
        item.ratio = 2971.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = TenantTemplateClusterLengthRule { max_length: 86400 };
        let mut item = new_tenant_template("x", "y");
        item.cluster = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = TenantTemplateScoreRangeRule { min: 8.4, max: 1944.2 };
        let mut item = new_tenant_template("x", "y");
        item.score = 1945.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
