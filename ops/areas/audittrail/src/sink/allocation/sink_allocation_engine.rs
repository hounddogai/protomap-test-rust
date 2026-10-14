use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the sink allocation calculation.
pub trait SinkAllocationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SinkAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkAllocationPortOffsetAdjuster {
    pub weight: f64,
}

impl SinkAllocationModifier for SinkAllocationPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &SinkAllocation, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSinkAllocationAdjuster {
    pub step: f64,
}

impl SinkAllocationModifier for RoundingSinkAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SinkAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSinkAllocationAdjuster {
    pub cap: f64,
}

impl SinkAllocationModifier for CapSinkAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SinkAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSinkAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SinkAllocationModifier for FlaggedSinkAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SinkAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the sink allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SinkAllocationEngine {
    pub steps: Vec<Rc<dyn SinkAllocationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SinkAllocationEngine {
    pub fn run(&mut self, item: &SinkAllocation) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[SinkAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_sink_allocation_engine(flags: &flags::Registry) -> SinkAllocationEngine {
    SinkAllocationEngine {
        steps: vec![
            Rc::new(SinkAllocationPortOffsetAdjuster { weight: 1.75 }) as Rc<dyn SinkAllocationModifier>,
            Rc::new(RoundingSinkAllocationAdjuster { step: 0.05 }),
            Rc::new(CapSinkAllocationAdjuster { cap: 38386.0 }),
            Rc::new(FlaggedSinkAllocationAdjuster {
                flags: flags.clone(),
                flag: "sink-allocation-pending".to_string(),
                percent: -5.4,
            }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 87542.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_sink_allocation_engine(&flags::new_registry());
        let item = new_sink_allocation("code-882", "label-512");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
