use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered overage template calculation.
pub trait TieredOverageTemplateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredOverageTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredOverageTemplateAdjuster {
    pub minimum: f64,
}

impl TieredOverageTemplateAdjuster for MinimumTieredOverageTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredOverageTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredOverageTemplateAdjuster {
    pub percent: f64,
}

impl TieredOverageTemplateAdjuster for PercentTieredOverageTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredOverageTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredOverageTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredOverageTemplateAdjuster for TieredTieredOverageTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredOverageTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredOverageTemplateAdjuster {
    pub cap: f64,
}

impl TieredOverageTemplateAdjuster for CapTieredOverageTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredOverageTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the tiered overage template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredOverageTemplateEngine {
    pub steps: Vec<Rc<dyn TieredOverageTemplateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredOverageTemplateEngine {
    pub fn compute(&mut self, item: &TieredOverageTemplate) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TieredOverageTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tiered_overage_template_engine() -> TieredOverageTemplateEngine {
    TieredOverageTemplateEngine {
        steps: vec![
            Rc::new(MinimumTieredOverageTemplateAdjuster { minimum: 29.5 }) as Rc<dyn TieredOverageTemplateAdjuster>,
            Rc::new(PercentTieredOverageTemplateAdjuster { percent: -18.1 }),
            Rc::new(TieredTieredOverageTemplateAdjuster {
                bounds: vec![8100.0, 8117.0],
                rates: vec![1.12, 1.13],
            }),
            Rc::new(CapTieredOverageTemplateAdjuster { cap: 48640.0 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 43335.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_overage_template_engine();
        let item = new_tiered_overage_template("code-326", "name-857");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
