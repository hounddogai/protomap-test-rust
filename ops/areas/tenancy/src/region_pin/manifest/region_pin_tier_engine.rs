use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the region pin tier calculation.
pub trait RegionPinTierModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionPinTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionPinTierAdjuster {
    pub minimum: f64,
}

impl RegionPinTierModifier for MinimumRegionPinTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionPinTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionPinTierAdjuster {
    pub step: f64,
}

impl RegionPinTierModifier for RoundingRegionPinTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionPinTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionPinTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionPinTierModifier for FlaggedRegionPinTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionPinTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionPinTierAdjuster {
    pub cap: f64,
}

impl RegionPinTierModifier for CapRegionPinTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionPinTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryRegionPinTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionPinTierModifier for CountryRegionPinTierAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &RegionPinTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the region pin tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionPinTierEngine {
    pub steps: Vec<Rc<dyn RegionPinTierModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionPinTierEngine {
    pub fn run(&mut self, item: &RegionPinTier) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RegionPinTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_region_pin_tier_engine(flags: &flags::Registry) -> RegionPinTierEngine {
    RegionPinTierEngine {
        steps: vec![
            Rc::new(MinimumRegionPinTierAdjuster { minimum: 13.2 }) as Rc<dyn RegionPinTierModifier>,
            Rc::new(RoundingRegionPinTierAdjuster { step: 1.0 }),
            Rc::new(FlaggedRegionPinTierAdjuster {
                flags: flags.clone(),
                flag: "region-pin-tier-regional".to_string(),
                percent: -7.7,
            }),
            Rc::new(CapRegionPinTierAdjuster { cap: 30947.0 }),
            Rc::new(CountryRegionPinTierAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.7),
                    ("retail".to_string(), 0.83),
                    ("central".to_string(), 0.95),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 49121.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_region_pin_tier_engine(&flags::new_registry());
        let item = new_region_pin_tier("code-517", "label-685");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
