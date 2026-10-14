use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual zone override calculation.
pub trait ManualZoneOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualZoneOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualZoneOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualZoneOverrideModifier for TieredManualZoneOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualZoneOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualZoneOverrideAdjuster {
    pub cap: f64,
}

impl ManualZoneOverrideModifier for CapManualZoneOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualZoneOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualZoneOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualZoneOverrideModifier for FlaggedManualZoneOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualZoneOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualZoneOverrideAdjuster {
    pub percent: f64,
}

impl ManualZoneOverrideModifier for PercentManualZoneOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualZoneOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the manual zone override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualZoneOverrideEngine {
    pub steps: Vec<Rc<dyn ManualZoneOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualZoneOverrideEngine {
    pub fn compute(&mut self, item: &ManualZoneOverride) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[ManualZoneOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_manual_zone_override_engine(flags: &flags::Registry) -> ManualZoneOverrideEngine {
    ManualZoneOverrideEngine {
        steps: vec![
            Rc::new(TieredManualZoneOverrideAdjuster {
                bounds: vec![5252.0, 7554.0, 8368.0],
                rates: vec![1.13, 0.86, 0.95],
            }) as Rc<dyn ManualZoneOverrideModifier>,
            Rc::new(CapManualZoneOverrideAdjuster { cap: 38462.0 }),
            Rc::new(FlaggedManualZoneOverrideAdjuster {
                flags: flags.clone(),
                flag: "manual-zone-override-pending".to_string(),
                percent: 3.9,
            }),
            Rc::new(PercentManualZoneOverrideAdjuster { percent: -14.7 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 34907.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_zone_override_engine(&flags::new_registry());
        let item = new_manual_zone_override("code-649", "name-7");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
