use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered segment adjustment calculation.
pub trait TieredSegmentAdjustmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredSegmentAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredSegmentAdjustmentAdjuster {
    pub percent: f64,
}

impl TieredSegmentAdjustmentAdjuster for PercentTieredSegmentAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredSegmentAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentAdjustmentPortOffsetAdjuster {
    pub weight: f64,
}

impl TieredSegmentAdjustmentAdjuster for TieredSegmentAdjustmentPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &TieredSegmentAdjustment, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleTieredSegmentAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredSegmentAdjustmentAdjuster for LocaleTieredSegmentAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &TieredSegmentAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredSegmentAdjustmentAdjuster {
    pub step: f64,
}

impl TieredSegmentAdjustmentAdjuster for RoundingTieredSegmentAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredSegmentAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredSegmentAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredSegmentAdjustmentAdjuster for FlaggedTieredSegmentAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredSegmentAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredSegmentAdjustmentAdjuster {
    pub minimum: f64,
}

impl TieredSegmentAdjustmentAdjuster for MinimumTieredSegmentAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredSegmentAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the tiered segment adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredSegmentAdjustmentEngine {
    pub steps: Vec<Rc<dyn TieredSegmentAdjustmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredSegmentAdjustmentEngine {
    pub fn run(&mut self, item: &TieredSegmentAdjustment) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[TieredSegmentAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tiered_segment_adjustment_engine(flags: &flags::Registry) -> TieredSegmentAdjustmentEngine {
    TieredSegmentAdjustmentEngine {
        steps: vec![
            Rc::new(PercentTieredSegmentAdjustmentAdjuster { percent: 10.5 })
                as Rc<dyn TieredSegmentAdjustmentAdjuster>,
            Rc::new(TieredSegmentAdjustmentPortOffsetAdjuster { weight: 0.52 }),
            Rc::new(LocaleTieredSegmentAdjustmentAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.09), ("online".to_string(), 0.92)]),
            }),
            Rc::new(RoundingTieredSegmentAdjustmentAdjuster { step: 5.0 }),
            Rc::new(FlaggedTieredSegmentAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "tiered-segment-adjustment-legacy".to_string(),
                percent: 3.1,
            }),
            Rc::new(MinimumTieredSegmentAdjustmentAdjuster { minimum: 38.8 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 97049.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_segment_adjustment_engine(&flags::new_registry());
        let item = new_tiered_segment_adjustment("code-896", "title-295");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
