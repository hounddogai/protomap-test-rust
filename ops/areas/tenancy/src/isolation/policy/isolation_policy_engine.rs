use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the isolation policy calculation.
pub trait IsolationPolicyFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IsolationPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumIsolationPolicyAdjuster {
    pub minimum: f64,
}

impl IsolationPolicyFactor for MinimumIsolationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &IsolationPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedIsolationPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IsolationPolicyFactor for FlaggedIsolationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IsolationPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIsolationPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IsolationPolicyFactor for TieredIsolationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IsolationPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIsolationPolicyAdjuster {
    pub step: f64,
}

impl IsolationPolicyFactor for RoundingIsolationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IsolationPolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the isolation policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IsolationPolicyEngine {
    pub steps: Vec<Rc<dyn IsolationPolicyFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IsolationPolicyEngine {
    pub fn compute(&mut self, item: &IsolationPolicy) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn IsolationPolicyFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[IsolationPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_isolation_policy_engine(flags: &flags::Registry) -> IsolationPolicyEngine {
    IsolationPolicyEngine {
        steps: vec![
            Rc::new(MinimumIsolationPolicyAdjuster { minimum: 38.3 }) as Rc<dyn IsolationPolicyFactor>,
            Rc::new(FlaggedIsolationPolicyAdjuster {
                flags: flags.clone(),
                flag: "isolation-policy-adaptive".to_string(),
                percent: -8.0,
            }),
            Rc::new(TieredIsolationPolicyAdjuster {
                bounds: vec![1158.0, 4147.0, 6378.0, 9139.0],
                rates: vec![0.83, 1.16, 1.02, 0.86],
            }),
            Rc::new(RoundingIsolationPolicyAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 61476.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_isolation_policy_engine(&flags::new_registry());
        let item = new_isolation_policy("code-217", "title-308");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
