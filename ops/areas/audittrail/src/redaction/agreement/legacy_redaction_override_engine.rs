use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy redaction override calculation.
pub trait LegacyRedactionOverrideStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyRedactionOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyRedactionOverrideAdjuster {
    pub minimum: f64,
}

impl LegacyRedactionOverrideStep for MinimumLegacyRedactionOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyRedactionOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyRedactionOverrideAdjuster {
    pub cap: f64,
}

impl LegacyRedactionOverrideStep for CapLegacyRedactionOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyRedactionOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyRedactionOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyRedactionOverrideStep for TieredLegacyRedactionOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyRedactionOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyRedactionOverrideAdjuster {
    pub percent: f64,
}

impl LegacyRedactionOverrideStep for PercentLegacyRedactionOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyRedactionOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the legacy redaction override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyRedactionOverrideEngine {
    pub steps: Vec<Rc<dyn LegacyRedactionOverrideStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyRedactionOverrideEngine {
    pub fn compute(&mut self, item: &LegacyRedactionOverride) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[LegacyRedactionOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_legacy_redaction_override_engine() -> LegacyRedactionOverrideEngine {
    LegacyRedactionOverrideEngine {
        steps: vec![
            Rc::new(MinimumLegacyRedactionOverrideAdjuster { minimum: 41.5 }) as Rc<dyn LegacyRedactionOverrideStep>,
            Rc::new(CapLegacyRedactionOverrideAdjuster { cap: 38656.0 }),
            Rc::new(TieredLegacyRedactionOverrideAdjuster {
                bounds: vec![3957.0, 7308.0],
                rates: vec![0.89, 0.9],
            }),
            Rc::new(PercentLegacyRedactionOverrideAdjuster { percent: -15.3 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 57947.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_redaction_override_engine();
        let item = new_legacy_redaction_override("code-141", "title-320");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
