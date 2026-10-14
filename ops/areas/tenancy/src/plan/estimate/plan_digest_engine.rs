use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the plan digest calculation.
pub trait PlanDigestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanDigestStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl PlanDigestStep for PlanDigestStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &PlanDigest, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PlanDigestStep for TieredPlanDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PlanDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPlanDigestAdjuster {
    pub minimum: f64,
}

impl PlanDigestStep for MinimumPlanDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PlanDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPlanDigestAdjuster {
    pub step: f64,
}

impl PlanDigestStep for RoundingPlanDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PlanDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitPlanDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PlanDigestStep for UnitPlanDigestAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &PlanDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPlanDigestAdjuster {
    pub cap: f64,
}

impl PlanDigestStep for CapPlanDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PlanDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the plan digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanDigestEngine {
    pub steps: Vec<Rc<dyn PlanDigestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanDigestEngine {
    pub fn run(&mut self, item: &PlanDigest) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[PlanDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_plan_digest_engine() -> PlanDigestEngine {
    PlanDigestEngine {
        steps: vec![
            Rc::new(PlanDigestStartsAtOffsetAdjuster { weight: 1.2 }) as Rc<dyn PlanDigestStep>,
            Rc::new(TieredPlanDigestAdjuster {
                bounds: vec![7856.0, 9784.0],
                rates: vec![1.15, 0.82],
            }),
            Rc::new(MinimumPlanDigestAdjuster { minimum: 45.4 }),
            Rc::new(RoundingPlanDigestAdjuster { step: 0.05 }),
            Rc::new(UnitPlanDigestAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.03)]),
            }),
            Rc::new(CapPlanDigestAdjuster { cap: 39649.0 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 75751.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_plan_digest_engine();
        let item = new_plan_digest("code-674", "name-584");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
