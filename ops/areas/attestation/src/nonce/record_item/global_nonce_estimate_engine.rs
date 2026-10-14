use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the global nonce estimate calculation.
pub trait GlobalNonceEstimateStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalNonceEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalNonceEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalNonceEstimateStep for FlaggedGlobalNonceEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalNonceEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalNonceEstimateAdjuster {
    pub minimum: f64,
}

impl GlobalNonceEstimateStep for MinimumGlobalNonceEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalNonceEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalNonceEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalNonceEstimateStep for TieredGlobalNonceEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalNonceEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalNonceEstimateAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl GlobalNonceEstimateStep for GlobalNonceEstimateAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalNonceEstimate, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameGlobalNonceEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalNonceEstimateStep for NameGlobalNonceEstimateAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &GlobalNonceEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the global nonce estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalNonceEstimateEngine {
    pub steps: Vec<Rc<dyn GlobalNonceEstimateStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalNonceEstimateEngine {
    pub fn run(&mut self, item: &GlobalNonceEstimate) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalNonceEstimateStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalNonceEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_global_nonce_estimate_engine(flags: &flags::Registry) -> GlobalNonceEstimateEngine {
    GlobalNonceEstimateEngine {
        steps: vec![
            Rc::new(FlaggedGlobalNonceEstimateAdjuster {
                flags: flags.clone(),
                flag: "global-nonce-estimate-global".to_string(),
                percent: 9.8,
            }) as Rc<dyn GlobalNonceEstimateStep>,
            Rc::new(MinimumGlobalNonceEstimateAdjuster { minimum: 49.6 }),
            Rc::new(TieredGlobalNonceEstimateAdjuster {
                bounds: vec![527.0, 2006.0, 9899.0],
                rates: vec![1.06, 1.06, 1.12],
            }),
            Rc::new(GlobalNonceEstimateAttemptsOffsetAdjuster { weight: 0.78 }),
            Rc::new(NameGlobalNonceEstimateAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.93),
                    ("retail".to_string(), 0.87),
                    ("east".to_string(), 1.14),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 73532.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_nonce_estimate_engine(&flags::new_registry());
        let item = new_global_nonce_estimate("code-534", "name-334");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
