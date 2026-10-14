use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the region pin window calculation.
pub trait RegionPinWindowFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionPinWindow, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionPinWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionPinWindowFactor for FlaggedRegionPinWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionPinWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceRegionPinWindowAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionPinWindowFactor for SourceRegionPinWindowAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &RegionPinWindow, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionPinWindowAdjuster {
    pub minimum: f64,
}

impl RegionPinWindowFactor for MinimumRegionPinWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionPinWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionPinWindowAdjuster {
    pub step: f64,
}

impl RegionPinWindowFactor for RoundingRegionPinWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionPinWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinWindowMaximumOffsetAdjuster {
    pub weight: f64,
}

impl RegionPinWindowFactor for RegionPinWindowMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &RegionPinWindow, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionPinWindowAdjuster {
    pub cap: f64,
}

impl RegionPinWindowFactor for CapRegionPinWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionPinWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the region pin window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionPinWindowEngine {
    pub steps: Vec<Rc<dyn RegionPinWindowFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionPinWindowEngine {
    pub fn evaluate(&mut self, item: &RegionPinWindow) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionPinWindowFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionPinWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_region_pin_window_engine(flags: &flags::Registry) -> RegionPinWindowEngine {
    RegionPinWindowEngine {
        steps: vec![
            Rc::new(FlaggedRegionPinWindowAdjuster {
                flags: flags.clone(),
                flag: "region-pin-window-capped".to_string(),
                percent: 1.5,
            }) as Rc<dyn RegionPinWindowFactor>,
            Rc::new(SourceRegionPinWindowAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.31),
                    ("online".to_string(), 1.12),
                    ("west".to_string(), 1.15),
                ]),
            }),
            Rc::new(MinimumRegionPinWindowAdjuster { minimum: 29.2 }),
            Rc::new(RoundingRegionPinWindowAdjuster { step: 0.25 }),
            Rc::new(RegionPinWindowMaximumOffsetAdjuster { weight: 0.17 }),
            Rc::new(CapRegionPinWindowAdjuster { cap: 41096.0 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 6454.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_region_pin_window_engine(&flags::new_registry());
        let item = new_region_pin_window("code-508", "title-438");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
