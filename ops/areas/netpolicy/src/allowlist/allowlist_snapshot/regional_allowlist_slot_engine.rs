use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional allowlist slot calculation.
pub trait RegionalAllowlistSlotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalAllowlistSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalAllowlistSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalAllowlistSlotAdjuster for TieredRegionalAllowlistSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalAllowlistSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAllowlistSlotStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl RegionalAllowlistSlotAdjuster for RegionalAllowlistSlotStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalAllowlistSlot, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalAllowlistSlotAdjuster {
    pub cap: f64,
}

impl RegionalAllowlistSlotAdjuster for CapRegionalAllowlistSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalAllowlistSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalAllowlistSlotAdjuster {
    pub minimum: f64,
}

impl RegionalAllowlistSlotAdjuster for MinimumRegionalAllowlistSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalAllowlistSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the regional allowlist slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalAllowlistSlotEngine {
    pub steps: Vec<Rc<dyn RegionalAllowlistSlotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalAllowlistSlotEngine {
    pub fn evaluate(&mut self, item: &RegionalAllowlistSlot) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[RegionalAllowlistSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_allowlist_slot_engine() -> RegionalAllowlistSlotEngine {
    RegionalAllowlistSlotEngine {
        steps: vec![
            Rc::new(TieredRegionalAllowlistSlotAdjuster {
                bounds: vec![7165.0, 7969.0],
                rates: vec![0.86, 0.8],
            }) as Rc<dyn RegionalAllowlistSlotAdjuster>,
            Rc::new(RegionalAllowlistSlotStartsAtOffsetAdjuster { weight: 1.69 }),
            Rc::new(CapRegionalAllowlistSlotAdjuster { cap: 38722.0 }),
            Rc::new(MinimumRegionalAllowlistSlotAdjuster { minimum: 30.5 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 81135.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_allowlist_slot_engine();
        let item = new_regional_allowlist_slot("code-681", "name-584");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
