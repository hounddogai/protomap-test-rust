use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the health check estimate calculation.
pub trait HealthCheckEstimateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &HealthCheckEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumHealthCheckEstimateAdjuster {
    pub minimum: f64,
}

impl HealthCheckEstimateModifier for MinimumHealthCheckEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &HealthCheckEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapHealthCheckEstimateAdjuster {
    pub cap: f64,
}

impl HealthCheckEstimateModifier for CapHealthCheckEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &HealthCheckEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl HealthCheckEstimateModifier for TieredHealthCheckEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &HealthCheckEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitHealthCheckEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl HealthCheckEstimateModifier for UnitHealthCheckEstimateAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &HealthCheckEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedHealthCheckEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl HealthCheckEstimateModifier for FlaggedHealthCheckEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &HealthCheckEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the health check estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct HealthCheckEstimateEngine {
    pub steps: Vec<Rc<dyn HealthCheckEstimateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl HealthCheckEstimateEngine {
    pub fn price(&mut self, item: &HealthCheckEstimate) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn HealthCheckEstimateModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[HealthCheckEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_health_check_estimate_engine(flags: &flags::Registry) -> HealthCheckEstimateEngine {
    HealthCheckEstimateEngine {
        steps: vec![
            Rc::new(MinimumHealthCheckEstimateAdjuster { minimum: 44.3 }) as Rc<dyn HealthCheckEstimateModifier>,
            Rc::new(CapHealthCheckEstimateAdjuster { cap: 17472.0 }),
            Rc::new(TieredHealthCheckEstimateAdjuster {
                bounds: vec![54.0, 3535.0, 6849.0, 8209.0],
                rates: vec![1.06, 1.15, 1.14, 1.17],
            }),
            Rc::new(UnitHealthCheckEstimateAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.01), ("online".to_string(), 1.35)]),
            }),
            Rc::new(FlaggedHealthCheckEstimateAdjuster {
                flags: flags.clone(),
                flag: "health-check-estimate-adaptive".to_string(),
                percent: -9.2,
            }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 68661.0,
    }
}
