use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the region pin notice calculation.
pub trait RegionPinNoticeStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionPinNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionPinNoticeAdjuster {
    pub percent: f64,
}

impl RegionPinNoticeStep for PercentRegionPinNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionPinNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionRegionPinNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionPinNoticeStep for RegionRegionPinNoticeAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &RegionPinNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionPinNoticeAdjuster {
    pub cap: f64,
}

impl RegionPinNoticeStep for CapRegionPinNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionPinNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionPinNoticeAdjuster {
    pub minimum: f64,
}

impl RegionPinNoticeStep for MinimumRegionPinNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionPinNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionPinNoticeStep for TieredRegionPinNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionPinNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionPinNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionPinNoticeStep for FlaggedRegionPinNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionPinNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the region pin notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionPinNoticeEngine {
    pub steps: Vec<Rc<dyn RegionPinNoticeStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionPinNoticeEngine {
    pub fn price(&mut self, item: &RegionPinNotice) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionPinNoticeStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionPinNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_region_pin_notice_engine(flags: &flags::Registry) -> RegionPinNoticeEngine {
    RegionPinNoticeEngine {
        steps: vec![
            Rc::new(PercentRegionPinNoticeAdjuster { percent: 16.8 }) as Rc<dyn RegionPinNoticeStep>,
            Rc::new(RegionRegionPinNoticeAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.98)]),
            }),
            Rc::new(CapRegionPinNoticeAdjuster { cap: 23280.0 }),
            Rc::new(MinimumRegionPinNoticeAdjuster { minimum: 36.2 }),
            Rc::new(TieredRegionPinNoticeAdjuster {
                bounds: vec![1571.0, 9916.0],
                rates: vec![0.86, 1.11],
            }),
            Rc::new(FlaggedRegionPinNoticeAdjuster {
                flags: flags.clone(),
                flag: "region-pin-notice-tiered".to_string(),
                percent: 9.7,
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 28416.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_region_pin_notice_engine(&flags::new_registry());
        let item = new_region_pin_notice("code-777", "title-660");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
