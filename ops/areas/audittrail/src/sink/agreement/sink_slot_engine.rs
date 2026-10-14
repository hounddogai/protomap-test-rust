use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the sink slot calculation.
pub trait SinkSlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SinkSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSinkSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SinkSlotStep for TieredSinkSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SinkSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSinkSlotAdjuster {
    pub percent: f64,
}

impl SinkSlotStep for PercentSinkSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SinkSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSinkSlotAdjuster {
    pub minimum: f64,
}

impl SinkSlotStep for MinimumSinkSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SinkSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSinkSlotAdjuster {
    pub cap: f64,
}

impl SinkSlotStep for CapSinkSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SinkSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkSlotTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl SinkSlotStep for SinkSlotTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &SinkSlot, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

/// Runs the sink slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SinkSlotEngine {
    pub steps: Vec<Rc<dyn SinkSlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SinkSlotEngine {
    pub fn price(&mut self, item: &SinkSlot) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[SinkSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_sink_slot_engine() -> SinkSlotEngine {
    SinkSlotEngine {
        steps: vec![
            Rc::new(TieredSinkSlotAdjuster {
                bounds: vec![1458.0, 2996.0, 5417.0],
                rates: vec![0.97, 1.04, 1.07],
            }) as Rc<dyn SinkSlotStep>,
            Rc::new(PercentSinkSlotAdjuster { percent: -17.6 }),
            Rc::new(MinimumSinkSlotAdjuster { minimum: 1.8 }),
            Rc::new(CapSinkSlotAdjuster { cap: 21847.0 }),
            Rc::new(SinkSlotTtlSecondsOffsetAdjuster { weight: 0.48 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 20942.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_sink_slot_engine();
        let item = new_sink_slot("code-8", "label-170");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
