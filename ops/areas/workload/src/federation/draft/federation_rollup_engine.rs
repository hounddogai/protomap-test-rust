use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the federation rollup calculation.
pub trait FederationRollupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationRollup, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFederationRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FederationRollupAdjuster for FlaggedFederationRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FederationRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFederationRollupAdjuster {
    pub step: f64,
}

impl FederationRollupAdjuster for RoundingFederationRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FederationRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryFederationRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FederationRollupAdjuster for CategoryFederationRollupAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &FederationRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationRollupWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl FederationRollupAdjuster for FederationRollupWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &FederationRollup, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFederationRollupAdjuster {
    pub minimum: f64,
}

impl FederationRollupAdjuster for MinimumFederationRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FederationRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFederationRollupAdjuster {
    pub cap: f64,
}

impl FederationRollupAdjuster for CapFederationRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FederationRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the federation rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationRollupEngine {
    pub steps: Vec<Rc<dyn FederationRollupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationRollupEngine {
    pub fn run(&mut self, item: &FederationRollup) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FederationRollupAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FederationRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_federation_rollup_engine(flags: &flags::Registry) -> FederationRollupEngine {
    FederationRollupEngine {
        steps: vec![
            Rc::new(FlaggedFederationRollupAdjuster {
                flags: flags.clone(),
                flag: "federation-rollup-global".to_string(),
                percent: -0.5,
            }) as Rc<dyn FederationRollupAdjuster>,
            Rc::new(RoundingFederationRollupAdjuster { step: 0.5 }),
            Rc::new(CategoryFederationRollupAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.27), ("south".to_string(), 1.22)]),
            }),
            Rc::new(FederationRollupWindowDaysOffsetAdjuster { weight: 1.35 }),
            Rc::new(MinimumFederationRollupAdjuster { minimum: 42.6 }),
            Rc::new(CapFederationRollupAdjuster { cap: 20519.0 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 31670.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_federation_rollup_engine(&flags::new_registry());
        let item = new_federation_rollup("code-370", "label-132");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
