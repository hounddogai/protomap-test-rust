use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the region pin quota calculation.
pub trait RegionPinQuotaFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionPinQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionPinQuotaAdjuster {
    pub percent: f64,
}

impl RegionPinQuotaFactor for PercentRegionPinQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionPinQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionPinQuotaAdjuster {
    pub cap: f64,
}

impl RegionPinQuotaFactor for CapRegionPinQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionPinQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinQuotaQuantityOffsetAdjuster {
    pub weight: f64,
}

impl RegionPinQuotaFactor for RegionPinQuotaQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &RegionPinQuota, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionPinQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionPinQuotaFactor for FlaggedRegionPinQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionPinQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionPinQuotaFactor for TieredRegionPinQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionPinQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the region pin quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionPinQuotaEngine {
    pub steps: Vec<Rc<dyn RegionPinQuotaFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionPinQuotaEngine {
    pub fn evaluate(&mut self, item: &RegionPinQuota) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionPinQuotaFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionPinQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_region_pin_quota_engine(flags: &flags::Registry) -> RegionPinQuotaEngine {
    RegionPinQuotaEngine {
        steps: vec![
            Rc::new(PercentRegionPinQuotaAdjuster { percent: 18.0 }) as Rc<dyn RegionPinQuotaFactor>,
            Rc::new(CapRegionPinQuotaAdjuster { cap: 26914.0 }),
            Rc::new(RegionPinQuotaQuantityOffsetAdjuster { weight: 1.25 }),
            Rc::new(FlaggedRegionPinQuotaAdjuster {
                flags: flags.clone(),
                flag: "region-pin-quota-strict".to_string(),
                percent: 1.7,
            }),
            Rc::new(TieredRegionPinQuotaAdjuster {
                bounds: vec![3884.0, 7782.0],
                rates: vec![0.89, 0.99],
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 80110.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_region_pin_quota_engine(&flags::new_registry());
        let item = new_region_pin_quota("code-654", "name-688");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
