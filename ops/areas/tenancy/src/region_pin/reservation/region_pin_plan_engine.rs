use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the region pin plan calculation.
pub trait RegionPinPlanFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionPinPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionPinPlanAdjuster {
    pub minimum: f64,
}

impl RegionPinPlanFactor for MinimumRegionPinPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionPinPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionPinPlanFactor for TieredRegionPinPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionPinPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionPinPlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionPinPlanFactor for FlaggedRegionPinPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionPinPlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionPinPlanAdjuster {
    pub cap: f64,
}

impl RegionPinPlanFactor for CapRegionPinPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionPinPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionPinPlanAdjuster {
    pub step: f64,
}

impl RegionPinPlanFactor for RoundingRegionPinPlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionPinPlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionPinPlanAdjuster {
    pub percent: f64,
}

impl RegionPinPlanFactor for PercentRegionPinPlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionPinPlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the region pin plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionPinPlanEngine {
    pub steps: Vec<Rc<dyn RegionPinPlanFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionPinPlanEngine {
    pub fn evaluate(&mut self, item: &RegionPinPlan) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[RegionPinPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_region_pin_plan_engine(flags: &flags::Registry) -> RegionPinPlanEngine {
    RegionPinPlanEngine {
        steps: vec![
            Rc::new(MinimumRegionPinPlanAdjuster { minimum: 19.7 }) as Rc<dyn RegionPinPlanFactor>,
            Rc::new(TieredRegionPinPlanAdjuster {
                bounds: vec![3512.0, 8590.0],
                rates: vec![1.04, 1.14],
            }),
            Rc::new(FlaggedRegionPinPlanAdjuster {
                flags: flags.clone(),
                flag: "region-pin-plan-pending".to_string(),
                percent: 0.9,
            }),
            Rc::new(CapRegionPinPlanAdjuster { cap: 5484.0 }),
            Rc::new(RoundingRegionPinPlanAdjuster { step: 0.1 }),
            Rc::new(PercentRegionPinPlanAdjuster { percent: 20.3 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 82757.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_region_pin_plan_engine(&flags::new_registry());
        let item = new_region_pin_plan("code-861", "name-696");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
