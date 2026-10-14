use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered plan profile calculation.
pub trait TieredPlanProfileAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredPlanProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredPlanProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredPlanProfileAdjuster for TieredTieredPlanProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredPlanProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredPlanProfileAdjuster {
    pub cap: f64,
}

impl TieredPlanProfileAdjuster for CapTieredPlanProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredPlanProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredPlanProfileAdjuster {
    pub minimum: f64,
}

impl TieredPlanProfileAdjuster for MinimumTieredPlanProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredPlanProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredPlanProfileAdjuster {
    pub step: f64,
}

impl TieredPlanProfileAdjuster for RoundingTieredPlanProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredPlanProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredPlanProfileAdjuster {
    pub percent: f64,
}

impl TieredPlanProfileAdjuster for PercentTieredPlanProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredPlanProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryTieredPlanProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredPlanProfileAdjuster for CategoryTieredPlanProfileAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &TieredPlanProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the tiered plan profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredPlanProfileEngine {
    pub steps: Vec<Rc<dyn TieredPlanProfileAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredPlanProfileEngine {
    pub fn run(&mut self, item: &TieredPlanProfile) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredPlanProfileAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredPlanProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tiered_plan_profile_engine() -> TieredPlanProfileEngine {
    TieredPlanProfileEngine {
        steps: vec![
            Rc::new(TieredTieredPlanProfileAdjuster {
                bounds: vec![1336.0, 6715.0, 7122.0, 7996.0],
                rates: vec![1.01, 1.02, 0.93, 0.95],
            }) as Rc<dyn TieredPlanProfileAdjuster>,
            Rc::new(CapTieredPlanProfileAdjuster { cap: 29747.0 }),
            Rc::new(MinimumTieredPlanProfileAdjuster { minimum: 30.3 }),
            Rc::new(RoundingTieredPlanProfileAdjuster { step: 0.1 }),
            Rc::new(PercentTieredPlanProfileAdjuster { percent: 7.2 }),
            Rc::new(CategoryTieredPlanProfileAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.31),
                    ("central".to_string(), 1.09),
                    ("north".to_string(), 1.3),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 61417.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_plan_profile_engine();
        let item = new_tiered_plan_profile("code-272", "label-602");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
