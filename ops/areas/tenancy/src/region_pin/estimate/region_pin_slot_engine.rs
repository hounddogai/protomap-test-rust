use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the region pin slot calculation.
pub trait RegionPinSlotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionPinSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceRegionPinSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionPinSlotModifier for NamespaceRegionPinSlotAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &RegionPinSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionPinSlotAdjuster {
    pub cap: f64,
}

impl RegionPinSlotModifier for CapRegionPinSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionPinSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSlotPortOffsetAdjuster {
    pub weight: f64,
}

impl RegionPinSlotModifier for RegionPinSlotPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &RegionPinSlot, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionPinSlotAdjuster {
    pub step: f64,
}

impl RegionPinSlotModifier for RoundingRegionPinSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionPinSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the region pin slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionPinSlotEngine {
    pub steps: Vec<Rc<dyn RegionPinSlotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionPinSlotEngine {
    pub fn evaluate(&mut self, item: &RegionPinSlot) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[RegionPinSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_region_pin_slot_engine() -> RegionPinSlotEngine {
    RegionPinSlotEngine {
        steps: vec![
            Rc::new(NamespaceRegionPinSlotAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.3),
                    ("south".to_string(), 1.2),
                    ("east".to_string(), 1.24),
                ]),
            }) as Rc<dyn RegionPinSlotModifier>,
            Rc::new(CapRegionPinSlotAdjuster { cap: 31465.0 }),
            Rc::new(RegionPinSlotPortOffsetAdjuster { weight: 1.4 }),
            Rc::new(RoundingRegionPinSlotAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 43003.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_region_pin_slot_engine();
        let item = new_region_pin_slot("code-565", "name-588");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
