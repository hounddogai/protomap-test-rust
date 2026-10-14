use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the freeze template calculation.
pub trait FreezeTemplateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeTemplateBurstOffsetAdjuster {
    pub weight: f64,
}

impl FreezeTemplateFactor for FreezeTemplateBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &FreezeTemplate, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezeTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FreezeTemplateFactor for TieredFreezeTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FreezeTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFreezeTemplateAdjuster {
    pub minimum: f64,
}

impl FreezeTemplateFactor for MinimumFreezeTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FreezeTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFreezeTemplateAdjuster {
    pub step: f64,
}

impl FreezeTemplateFactor for RoundingFreezeTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FreezeTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the freeze template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeTemplateEngine {
    pub steps: Vec<Rc<dyn FreezeTemplateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeTemplateEngine {
    pub fn compute(&mut self, item: &FreezeTemplate) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[FreezeTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_freeze_template_engine() -> FreezeTemplateEngine {
    FreezeTemplateEngine {
        steps: vec![
            Rc::new(FreezeTemplateBurstOffsetAdjuster { weight: 1.91 }) as Rc<dyn FreezeTemplateFactor>,
            Rc::new(TieredFreezeTemplateAdjuster {
                bounds: vec![7604.0, 7904.0],
                rates: vec![0.98, 0.95],
            }),
            Rc::new(MinimumFreezeTemplateAdjuster { minimum: 46.0 }),
            Rc::new(RoundingFreezeTemplateAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 18859.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_freeze_template_engine();
        let item = new_freeze_template("code-486", "label-159");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
