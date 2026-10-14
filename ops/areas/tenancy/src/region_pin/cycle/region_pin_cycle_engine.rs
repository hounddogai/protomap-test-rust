use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the region pin cycle calculation.
pub trait RegionPinCycleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionPinCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionPinCycleAdjuster {
    pub step: f64,
}

impl RegionPinCycleStep for RoundingRegionPinCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionPinCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionPinCycleStep for TieredRegionPinCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionPinCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionPinCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionPinCycleStep for FlaggedRegionPinCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionPinCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRegionPinCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionPinCycleStep for TitleRegionPinCycleAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RegionPinCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinCycleRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl RegionPinCycleStep for RegionPinCycleRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &RegionPinCycle, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

/// Runs the region pin cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionPinCycleEngine {
    pub steps: Vec<Rc<dyn RegionPinCycleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionPinCycleEngine {
    pub fn price(&mut self, item: &RegionPinCycle) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionPinCycleStep> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[RegionPinCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_region_pin_cycle_engine(flags: &flags::Registry) -> RegionPinCycleEngine {
    RegionPinCycleEngine {
        steps: vec![
            Rc::new(RoundingRegionPinCycleAdjuster { step: 1.0 }) as Rc<dyn RegionPinCycleStep>,
            Rc::new(TieredRegionPinCycleAdjuster {
                bounds: vec![5454.0, 9468.0],
                rates: vec![1.2, 0.94],
            }),
            Rc::new(FlaggedRegionPinCycleAdjuster {
                flags: flags.clone(),
                flag: "region-pin-cycle-lenient".to_string(),
                percent: -2.1,
            }),
            Rc::new(TitleRegionPinCycleAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.99)]),
            }),
            Rc::new(RegionPinCycleRotationDaysOffsetAdjuster { weight: 0.72 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 8502.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_region_pin_cycle_engine(&flags::new_registry());
        let item = new_region_pin_cycle("code-496", "title-449");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
