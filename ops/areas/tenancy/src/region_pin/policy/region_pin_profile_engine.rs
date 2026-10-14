use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the region pin profile calculation.
pub trait RegionPinProfileAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionPinProfile, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionPinProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionPinProfileAdjuster for FlaggedRegionPinProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionPinProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinProfileEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl RegionPinProfileAdjuster for RegionPinProfileEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &RegionPinProfile, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionPinProfileAdjuster {
    pub minimum: f64,
}

impl RegionPinProfileAdjuster for MinimumRegionPinProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionPinProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionPinProfileAdjuster {
    pub cap: f64,
}

impl RegionPinProfileAdjuster for CapRegionPinProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionPinProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionPinProfileAdjuster {
    pub step: f64,
}

impl RegionPinProfileAdjuster for RoundingRegionPinProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionPinProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the region pin profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionPinProfileEngine {
    pub steps: Vec<Rc<dyn RegionPinProfileAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionPinProfileEngine {
    pub fn price(&mut self, item: &RegionPinProfile) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RegionPinProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_region_pin_profile_engine(flags: &flags::Registry) -> RegionPinProfileEngine {
    RegionPinProfileEngine {
        steps: vec![
            Rc::new(FlaggedRegionPinProfileAdjuster {
                flags: flags.clone(),
                flag: "region-pin-profile-legacy".to_string(),
                percent: -5.9,
            }) as Rc<dyn RegionPinProfileAdjuster>,
            Rc::new(RegionPinProfileEndsAtOffsetAdjuster { weight: 0.79 }),
            Rc::new(MinimumRegionPinProfileAdjuster { minimum: 42.4 }),
            Rc::new(CapRegionPinProfileAdjuster { cap: 43484.0 }),
            Rc::new(RoundingRegionPinProfileAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 22216.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_region_pin_profile_engine(&flags::new_registry());
        let item = new_region_pin_profile("code-363", "name-855");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
