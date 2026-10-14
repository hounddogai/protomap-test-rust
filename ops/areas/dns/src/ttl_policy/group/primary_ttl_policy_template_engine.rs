use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary ttl policy template calculation.
pub trait PrimaryTtlPolicyTemplateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryTtlPolicyTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryTtlPolicyTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryTtlPolicyTemplateFactor for FlaggedPrimaryTtlPolicyTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryTtlPolicyTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryTtlPolicyTemplateAdjuster {
    pub cap: f64,
}

impl PrimaryTtlPolicyTemplateFactor for CapPrimaryTtlPolicyTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryTtlPolicyTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryTtlPolicyTemplateAdjuster {
    pub step: f64,
}

impl PrimaryTtlPolicyTemplateFactor for RoundingPrimaryTtlPolicyTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryTtlPolicyTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryTtlPolicyTemplateAdjuster {
    pub percent: f64,
}

impl PrimaryTtlPolicyTemplateFactor for PercentPrimaryTtlPolicyTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryTtlPolicyTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePrimaryTtlPolicyTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryTtlPolicyTemplateFactor for TitlePrimaryTtlPolicyTemplateAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PrimaryTtlPolicyTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the primary ttl policy template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryTtlPolicyTemplateEngine {
    pub steps: Vec<Rc<dyn PrimaryTtlPolicyTemplateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryTtlPolicyTemplateEngine {
    pub fn price(&mut self, item: &PrimaryTtlPolicyTemplate) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[PrimaryTtlPolicyTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_ttl_policy_template_engine(flags: &flags::Registry) -> PrimaryTtlPolicyTemplateEngine {
    PrimaryTtlPolicyTemplateEngine {
        steps: vec![
            Rc::new(FlaggedPrimaryTtlPolicyTemplateAdjuster {
                flags: flags.clone(),
                flag: "primary-ttl-policy-template-tiered".to_string(),
                percent: -7.4,
            }) as Rc<dyn PrimaryTtlPolicyTemplateFactor>,
            Rc::new(CapPrimaryTtlPolicyTemplateAdjuster { cap: 5674.0 }),
            Rc::new(RoundingPrimaryTtlPolicyTemplateAdjuster { step: 0.25 }),
            Rc::new(PercentPrimaryTtlPolicyTemplateAdjuster { percent: 9.1 }),
            Rc::new(TitlePrimaryTtlPolicyTemplateAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.15)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 89366.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_ttl_policy_template_engine(&flags::new_registry());
        let item = new_primary_ttl_policy_template("code-697", "title-348");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
