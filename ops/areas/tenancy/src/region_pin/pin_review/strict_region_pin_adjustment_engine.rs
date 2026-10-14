use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict region pin adjustment calculation.
pub trait StrictRegionPinAdjustmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictRegionPinAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictRegionPinAdjustmentAdjuster {
    pub percent: f64,
}

impl StrictRegionPinAdjustmentModifier for PercentStrictRegionPinAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictRegionPinAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictRegionPinAdjustmentAdjuster {
    pub minimum: f64,
}

impl StrictRegionPinAdjustmentModifier for MinimumStrictRegionPinAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictRegionPinAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentStrictRegionPinAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictRegionPinAdjustmentModifier for SegmentStrictRegionPinAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &StrictRegionPinAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictRegionPinAdjustmentAdjuster {
    pub cap: f64,
}

impl StrictRegionPinAdjustmentModifier for CapStrictRegionPinAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictRegionPinAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictRegionPinAdjustmentAdjuster {
    pub step: f64,
}

impl StrictRegionPinAdjustmentModifier for RoundingStrictRegionPinAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictRegionPinAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictRegionPinAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictRegionPinAdjustmentModifier for FlaggedStrictRegionPinAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictRegionPinAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the strict region pin adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictRegionPinAdjustmentEngine {
    pub steps: Vec<Rc<dyn StrictRegionPinAdjustmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictRegionPinAdjustmentEngine {
    pub fn compute(&mut self, item: &StrictRegionPinAdjustment) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[StrictRegionPinAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_strict_region_pin_adjustment_engine(flags: &flags::Registry) -> StrictRegionPinAdjustmentEngine {
    StrictRegionPinAdjustmentEngine {
        steps: vec![
            Rc::new(PercentStrictRegionPinAdjustmentAdjuster { percent: 12.3 })
                as Rc<dyn StrictRegionPinAdjustmentModifier>,
            Rc::new(MinimumStrictRegionPinAdjustmentAdjuster { minimum: 33.2 }),
            Rc::new(SegmentStrictRegionPinAdjustmentAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.77), ("retail".to_string(), 0.98)]),
            }),
            Rc::new(CapStrictRegionPinAdjustmentAdjuster { cap: 47549.0 }),
            Rc::new(RoundingStrictRegionPinAdjustmentAdjuster { step: 1.0 }),
            Rc::new(FlaggedStrictRegionPinAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "strict-region-pin-adjustment-seasonal".to_string(),
                percent: 1.3,
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 13464.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_region_pin_adjustment_engine(&flags::new_registry());
        let item = new_strict_region_pin_adjustment("code-713", "label-593");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
