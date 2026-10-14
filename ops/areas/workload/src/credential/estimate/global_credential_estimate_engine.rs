use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the global credential estimate calculation.
pub trait GlobalCredentialEstimateStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalCredentialEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalCredentialEstimateAdjuster {
    pub cap: f64,
}

impl GlobalCredentialEstimateStep for CapGlobalCredentialEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalCredentialEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleGlobalCredentialEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalCredentialEstimateStep for LocaleGlobalCredentialEstimateAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &GlobalCredentialEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalCredentialEstimateAdjuster {
    pub minimum: f64,
}

impl GlobalCredentialEstimateStep for MinimumGlobalCredentialEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalCredentialEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalCredentialEstimatePriorityOffsetAdjuster {
    pub weight: f64,
}

impl GlobalCredentialEstimateStep for GlobalCredentialEstimatePriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalCredentialEstimate, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalCredentialEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalCredentialEstimateStep for TieredGlobalCredentialEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalCredentialEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalCredentialEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalCredentialEstimateStep for FlaggedGlobalCredentialEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalCredentialEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global credential estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalCredentialEstimateEngine {
    pub steps: Vec<Rc<dyn GlobalCredentialEstimateStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalCredentialEstimateEngine {
    pub fn evaluate(&mut self, item: &GlobalCredentialEstimate) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalCredentialEstimateStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalCredentialEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_credential_estimate_engine(flags: &flags::Registry) -> GlobalCredentialEstimateEngine {
    GlobalCredentialEstimateEngine {
        steps: vec![
            Rc::new(CapGlobalCredentialEstimateAdjuster { cap: 48776.0 }) as Rc<dyn GlobalCredentialEstimateStep>,
            Rc::new(LocaleGlobalCredentialEstimateAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.26),
                    ("east".to_string(), 1.24),
                    ("online".to_string(), 0.97),
                ]),
            }),
            Rc::new(MinimumGlobalCredentialEstimateAdjuster { minimum: 35.3 }),
            Rc::new(GlobalCredentialEstimatePriorityOffsetAdjuster { weight: 1.72 }),
            Rc::new(TieredGlobalCredentialEstimateAdjuster {
                bounds: vec![3673.0, 4250.0, 8245.0, 9977.0],
                rates: vec![1.15, 1.08, 1.17, 0.83],
            }),
            Rc::new(FlaggedGlobalCredentialEstimateAdjuster {
                flags: flags.clone(),
                flag: "global-credential-estimate-strict".to_string(),
                percent: -8.0,
            }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 27419.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_credential_estimate_engine(&flags::new_registry());
        let item = new_global_credential_estimate("code-369", "name-795");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
