use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the fallback region pin plan calculation.
pub trait FallbackRegionPinPlanFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackRegionPinPlan, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackRegionPinPlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackRegionPinPlanFactor for FlaggedFallbackRegionPinPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackRegionPinPlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackRegionPinPlanAdjuster {
    pub percent: f64,
}

impl FallbackRegionPinPlanFactor for PercentFallbackRegionPinPlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackRegionPinPlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRegionPinPlanFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl FallbackRegionPinPlanFactor for FallbackRegionPinPlanFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackRegionPinPlan, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackRegionPinPlanAdjuster {
    pub minimum: f64,
}

impl FallbackRegionPinPlanFactor for MinimumFallbackRegionPinPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackRegionPinPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackRegionPinPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackRegionPinPlanFactor for TieredFallbackRegionPinPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackRegionPinPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the fallback region pin plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackRegionPinPlanEngine {
    pub steps: Vec<Rc<dyn FallbackRegionPinPlanFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackRegionPinPlanEngine {
    pub fn run(&mut self, item: &FallbackRegionPinPlan) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackRegionPinPlanFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackRegionPinPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_fallback_region_pin_plan_engine(flags: &flags::Registry) -> FallbackRegionPinPlanEngine {
    FallbackRegionPinPlanEngine {
        steps: vec![
            Rc::new(FlaggedFallbackRegionPinPlanAdjuster {
                flags: flags.clone(),
                flag: "fallback-region-pin-plan-lenient".to_string(),
                percent: -4.1,
            }) as Rc<dyn FallbackRegionPinPlanFactor>,
            Rc::new(PercentFallbackRegionPinPlanAdjuster { percent: 13.9 }),
            Rc::new(FallbackRegionPinPlanFloorValueOffsetAdjuster { weight: 0.39 }),
            Rc::new(MinimumFallbackRegionPinPlanAdjuster { minimum: 33.5 }),
            Rc::new(TieredFallbackRegionPinPlanAdjuster {
                bounds: vec![260.0, 4233.0, 6128.0],
                rates: vec![0.91, 1.13, 0.81],
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 68812.1,
    }
}
