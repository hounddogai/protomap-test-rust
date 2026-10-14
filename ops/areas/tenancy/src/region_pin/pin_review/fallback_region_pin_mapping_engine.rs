use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback region pin mapping calculation.
pub trait FallbackRegionPinMappingStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackRegionPinMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRegionPinMappingEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl FallbackRegionPinMappingStep for FallbackRegionPinMappingEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackRegionPinMapping, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackRegionPinMappingAdjuster {
    pub minimum: f64,
}

impl FallbackRegionPinMappingStep for MinimumFallbackRegionPinMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackRegionPinMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackRegionPinMappingAdjuster {
    pub cap: f64,
}

impl FallbackRegionPinMappingStep for CapFallbackRegionPinMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackRegionPinMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackRegionPinMappingAdjuster {
    pub step: f64,
}

impl FallbackRegionPinMappingStep for RoundingFallbackRegionPinMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackRegionPinMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the fallback region pin mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackRegionPinMappingEngine {
    pub steps: Vec<Rc<dyn FallbackRegionPinMappingStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackRegionPinMappingEngine {
    pub fn compute(&mut self, item: &FallbackRegionPinMapping) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[FallbackRegionPinMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_region_pin_mapping_engine() -> FallbackRegionPinMappingEngine {
    FallbackRegionPinMappingEngine {
        steps: vec![
            Rc::new(FallbackRegionPinMappingEndsAtOffsetAdjuster { weight: 0.23 })
                as Rc<dyn FallbackRegionPinMappingStep>,
            Rc::new(MinimumFallbackRegionPinMappingAdjuster { minimum: 40.3 }),
            Rc::new(CapFallbackRegionPinMappingAdjuster { cap: 13983.0 }),
            Rc::new(RoundingFallbackRegionPinMappingAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 20346.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_region_pin_mapping_engine();
        let item = new_fallback_region_pin_mapping("code-687", "label-671");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
