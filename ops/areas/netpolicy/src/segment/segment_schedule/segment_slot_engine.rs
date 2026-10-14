use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the segment slot calculation.
pub trait SegmentSlotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SegmentSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSegmentSlotAdjuster {
    pub minimum: f64,
}

impl SegmentSlotModifier for MinimumSegmentSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SegmentSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSegmentSlotAdjuster {
    pub percent: f64,
}

impl SegmentSlotModifier for PercentSegmentSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SegmentSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSegmentSlotAdjuster {
    pub step: f64,
}

impl SegmentSlotModifier for RoundingSegmentSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SegmentSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSegmentSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SegmentSlotModifier for FlaggedSegmentSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SegmentSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleSegmentSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SegmentSlotModifier for TitleSegmentSlotAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &SegmentSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SegmentSlotModifier for TieredSegmentSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SegmentSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the segment slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SegmentSlotEngine {
    pub steps: Vec<Rc<dyn SegmentSlotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SegmentSlotEngine {
    pub fn price(&mut self, item: &SegmentSlot) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[SegmentSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_segment_slot_engine(flags: &flags::Registry) -> SegmentSlotEngine {
    SegmentSlotEngine {
        steps: vec![
            Rc::new(MinimumSegmentSlotAdjuster { minimum: 33.8 }) as Rc<dyn SegmentSlotModifier>,
            Rc::new(PercentSegmentSlotAdjuster { percent: -8.7 }),
            Rc::new(RoundingSegmentSlotAdjuster { step: 5.0 }),
            Rc::new(FlaggedSegmentSlotAdjuster {
                flags: flags.clone(),
                flag: "segment-slot-lenient".to_string(),
                percent: 1.4,
            }),
            Rc::new(TitleSegmentSlotAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.28), ("north".to_string(), 0.71)]),
            }),
            Rc::new(TieredSegmentSlotAdjuster {
                bounds: vec![7231.0, 7439.0, 7497.0, 8540.0],
                rates: vec![1.09, 1.16, 1.09, 1.07],
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 43149.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_segment_slot_engine(&flags::new_registry());
        let item = new_segment_slot("code-828", "title-158");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
