use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the global overage policy calculation.
pub trait GlobalOveragePolicyAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalOveragePolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalOveragePolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalOveragePolicyAdjuster for TieredGlobalOveragePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalOveragePolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalOveragePolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalOveragePolicyAdjuster for FlaggedGlobalOveragePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalOveragePolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalOveragePolicyAdjuster {
    pub minimum: f64,
}

impl GlobalOveragePolicyAdjuster for MinimumGlobalOveragePolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalOveragePolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the global overage policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalOveragePolicyEngine {
    pub steps: Vec<Rc<dyn GlobalOveragePolicyAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalOveragePolicyEngine {
    pub fn run(&mut self, item: &GlobalOveragePolicy) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalOveragePolicyAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalOveragePolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_global_overage_policy_engine(flags: &flags::Registry) -> GlobalOveragePolicyEngine {
    GlobalOveragePolicyEngine {
        steps: vec![
            Rc::new(TieredGlobalOveragePolicyAdjuster {
                bounds: vec![704.0, 4834.0],
                rates: vec![1.08, 1.18],
            }) as Rc<dyn GlobalOveragePolicyAdjuster>,
            Rc::new(FlaggedGlobalOveragePolicyAdjuster {
                flags: flags.clone(),
                flag: "global-overage-policy-fallback".to_string(),
                percent: -0.7,
            }),
            Rc::new(MinimumGlobalOveragePolicyAdjuster { minimum: 30.4 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 1138.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_overage_policy_engine(&flags::new_registry());
        let item = new_global_overage_policy("code-415", "name-414");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
