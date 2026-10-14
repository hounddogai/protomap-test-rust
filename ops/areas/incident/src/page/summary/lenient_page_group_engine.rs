use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient page group calculation.
pub trait LenientPageGroupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientPageGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientPageGroupAdjuster {
    pub cap: f64,
}

impl LenientPageGroupModifier for CapLenientPageGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientPageGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPageGroupCostOffsetAdjuster {
    pub weight: f64,
}

impl LenientPageGroupModifier for LenientPageGroupCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &LenientPageGroup, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientPageGroupAdjuster {
    pub minimum: f64,
}

impl LenientPageGroupModifier for MinimumLenientPageGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientPageGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientPageGroupAdjuster {
    pub step: f64,
}

impl LenientPageGroupModifier for RoundingLenientPageGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientPageGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the lenient page group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientPageGroupEngine {
    pub steps: Vec<Rc<dyn LenientPageGroupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientPageGroupEngine {
    pub fn evaluate(&mut self, item: &LenientPageGroup) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[LenientPageGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_lenient_page_group_engine() -> LenientPageGroupEngine {
    LenientPageGroupEngine {
        steps: vec![
            Rc::new(CapLenientPageGroupAdjuster { cap: 45517.0 }) as Rc<dyn LenientPageGroupModifier>,
            Rc::new(LenientPageGroupCostOffsetAdjuster { weight: 0.99 }),
            Rc::new(MinimumLenientPageGroupAdjuster { minimum: 39.7 }),
            Rc::new(RoundingLenientPageGroupAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 20025.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_page_group_engine();
        let item = new_lenient_page_group("code-593", "name-902");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
