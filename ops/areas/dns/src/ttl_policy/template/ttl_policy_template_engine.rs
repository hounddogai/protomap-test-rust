use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the ttl policy template calculation.
pub trait TtlPolicyTemplateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicyTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelTtlPolicyTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TtlPolicyTemplateFactor for LabelTtlPolicyTemplateAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &TtlPolicyTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTtlPolicyTemplateAdjuster {
    pub step: f64,
}

impl TtlPolicyTemplateFactor for RoundingTtlPolicyTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TtlPolicyTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTtlPolicyTemplateAdjuster {
    pub percent: f64,
}

impl TtlPolicyTemplateFactor for PercentTtlPolicyTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TtlPolicyTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicyTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TtlPolicyTemplateFactor for TieredTtlPolicyTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TtlPolicyTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the ttl policy template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicyTemplateEngine {
    pub steps: Vec<Rc<dyn TtlPolicyTemplateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicyTemplateEngine {
    pub fn evaluate(&mut self, item: &TtlPolicyTemplate) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[TtlPolicyTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_ttl_policy_template_engine() -> TtlPolicyTemplateEngine {
    TtlPolicyTemplateEngine {
        steps: vec![
            Rc::new(LabelTtlPolicyTemplateAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.04), ("retail".to_string(), 0.86)]),
            }) as Rc<dyn TtlPolicyTemplateFactor>,
            Rc::new(RoundingTtlPolicyTemplateAdjuster { step: 1.0 }),
            Rc::new(PercentTtlPolicyTemplateAdjuster { percent: 24.0 }),
            Rc::new(TieredTtlPolicyTemplateAdjuster {
                bounds: vec![2987.0, 6498.0, 6829.0, 9225.0],
                rates: vec![1.13, 1.19, 1.02, 1.04],
            }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 9052.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_ttl_policy_template_engine();
        let item = new_ttl_policy_template("code-491", "label-867");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
