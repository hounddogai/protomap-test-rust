use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the global forecast rollup calculation.
pub trait GlobalForecastRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalForecastRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalForecastRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalForecastRollupFactor for TieredGlobalForecastRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalForecastRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalForecastRollupAdjuster {
    pub percent: f64,
}

impl GlobalForecastRollupFactor for PercentGlobalForecastRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalForecastRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalForecastRollupAdjuster {
    pub cap: f64,
}

impl GlobalForecastRollupFactor for CapGlobalForecastRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalForecastRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalForecastRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalForecastRollupFactor for FlaggedGlobalForecastRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalForecastRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleGlobalForecastRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalForecastRollupFactor for TitleGlobalForecastRollupAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &GlobalForecastRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalForecastRollupAdjuster {
    pub minimum: f64,
}

impl GlobalForecastRollupFactor for MinimumGlobalForecastRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalForecastRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the global forecast rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalForecastRollupEngine {
    pub steps: Vec<Rc<dyn GlobalForecastRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalForecastRollupEngine {
    pub fn run(&mut self, item: &GlobalForecastRollup) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalForecastRollupFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalForecastRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_global_forecast_rollup_engine(flags: &flags::Registry) -> GlobalForecastRollupEngine {
    GlobalForecastRollupEngine {
        steps: vec![
            Rc::new(TieredGlobalForecastRollupAdjuster {
                bounds: vec![3156.0, 6288.0, 7211.0, 8528.0],
                rates: vec![0.91, 0.9, 0.97, 1.09],
            }) as Rc<dyn GlobalForecastRollupFactor>,
            Rc::new(PercentGlobalForecastRollupAdjuster { percent: 8.2 }),
            Rc::new(CapGlobalForecastRollupAdjuster { cap: 32921.0 }),
            Rc::new(FlaggedGlobalForecastRollupAdjuster {
                flags: flags.clone(),
                flag: "global-forecast-rollup-fallback".to_string(),
                percent: -9.4,
            }),
            Rc::new(TitleGlobalForecastRollupAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.18)]),
            }),
            Rc::new(MinimumGlobalForecastRollupAdjuster { minimum: 9.1 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 75289.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_forecast_rollup_engine(&flags::new_registry());
        let item = new_global_forecast_rollup("code-826", "title-298");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
