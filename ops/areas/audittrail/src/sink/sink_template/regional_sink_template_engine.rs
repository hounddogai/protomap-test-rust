use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional sink template calculation.
pub trait RegionalSinkTemplateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalSinkTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalSinkTemplateAdjuster {
    pub step: f64,
}

impl RegionalSinkTemplateFactor for RoundingRegionalSinkTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalSinkTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSinkTemplateSequenceOffsetAdjuster {
    pub weight: f64,
}

impl RegionalSinkTemplateFactor for RegionalSinkTemplateSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalSinkTemplate, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalSinkTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalSinkTemplateFactor for TieredRegionalSinkTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalSinkTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the regional sink template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalSinkTemplateEngine {
    pub steps: Vec<Rc<dyn RegionalSinkTemplateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalSinkTemplateEngine {
    pub fn price(&mut self, item: &RegionalSinkTemplate) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[RegionalSinkTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_sink_template_engine() -> RegionalSinkTemplateEngine {
    RegionalSinkTemplateEngine {
        steps: vec![
            Rc::new(RoundingRegionalSinkTemplateAdjuster { step: 5.0 }) as Rc<dyn RegionalSinkTemplateFactor>,
            Rc::new(RegionalSinkTemplateSequenceOffsetAdjuster { weight: 1.59 }),
            Rc::new(TieredRegionalSinkTemplateAdjuster {
                bounds: vec![160.0, 2617.0, 4374.0, 8283.0],
                rates: vec![1.15, 0.92, 0.89, 0.83],
            }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 37298.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_sink_template_engine();
        let item = new_regional_sink_template("code-580", "label-348");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
