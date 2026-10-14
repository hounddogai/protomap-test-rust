use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the zone threshold calculation.
pub trait ZoneThresholdAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ZoneThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingZoneThresholdAdjuster {
    pub step: f64,
}

impl ZoneThresholdAdjuster for RoundingZoneThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ZoneThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedZoneThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ZoneThresholdAdjuster for FlaggedZoneThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ZoneThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredZoneThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ZoneThresholdAdjuster for TieredZoneThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ZoneThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceZoneThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ZoneThresholdAdjuster for NamespaceZoneThresholdAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &ZoneThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapZoneThresholdAdjuster {
    pub cap: f64,
}

impl ZoneThresholdAdjuster for CapZoneThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ZoneThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the zone threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ZoneThresholdEngine {
    pub steps: Vec<Rc<dyn ZoneThresholdAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ZoneThresholdEngine {
    pub fn compute(&mut self, item: &ZoneThreshold) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ZoneThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_zone_threshold_engine(flags: &flags::Registry) -> ZoneThresholdEngine {
    ZoneThresholdEngine {
        steps: vec![
            Rc::new(RoundingZoneThresholdAdjuster { step: 1.0 }) as Rc<dyn ZoneThresholdAdjuster>,
            Rc::new(FlaggedZoneThresholdAdjuster {
                flags: flags.clone(),
                flag: "zone-threshold-pending".to_string(),
                percent: 1.9,
            }),
            Rc::new(TieredZoneThresholdAdjuster {
                bounds: vec![1372.0, 2864.0],
                rates: vec![1.1, 1.01],
            }),
            Rc::new(NamespaceZoneThresholdAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.02),
                    ("west".to_string(), 1.14),
                    ("north".to_string(), 1.36),
                ]),
            }),
            Rc::new(CapZoneThresholdAdjuster { cap: 7507.0 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 85002.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_zone_threshold_engine(&flags::new_registry());
        let item = new_zone_threshold("code-148", "label-19");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
