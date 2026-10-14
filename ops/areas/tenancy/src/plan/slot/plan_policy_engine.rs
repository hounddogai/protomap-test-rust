use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the plan policy calculation.
pub trait PlanPolicyStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPlanPolicyAdjuster {
    pub minimum: f64,
}

impl PlanPolicyStep for MinimumPlanPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PlanPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanPolicyVersionOffsetAdjuster {
    pub weight: f64,
}

impl PlanPolicyStep for PlanPolicyVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &PlanPolicy, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PlanPolicyStep for TieredPlanPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PlanPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPlanPolicyAdjuster {
    pub cap: f64,
}

impl PlanPolicyStep for CapPlanPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PlanPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPlanPolicyAdjuster {
    pub percent: f64,
}

impl PlanPolicyStep for PercentPlanPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PlanPolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPlanPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PlanPolicyStep for FlaggedPlanPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PlanPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the plan policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanPolicyEngine {
    pub steps: Vec<Rc<dyn PlanPolicyStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanPolicyEngine {
    pub fn evaluate(&mut self, item: &PlanPolicy) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[PlanPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_plan_policy_engine(flags: &flags::Registry) -> PlanPolicyEngine {
    PlanPolicyEngine {
        steps: vec![
            Rc::new(MinimumPlanPolicyAdjuster { minimum: 4.5 }) as Rc<dyn PlanPolicyStep>,
            Rc::new(PlanPolicyVersionOffsetAdjuster { weight: 0.36 }),
            Rc::new(TieredPlanPolicyAdjuster {
                bounds: vec![1122.0, 6173.0, 6449.0, 9041.0],
                rates: vec![0.87, 0.84, 0.86, 1.05],
            }),
            Rc::new(CapPlanPolicyAdjuster { cap: 40945.0 }),
            Rc::new(PercentPlanPolicyAdjuster { percent: -18.9 }),
            Rc::new(FlaggedPlanPolicyAdjuster {
                flags: flags.clone(),
                flag: "plan-policy-legacy".to_string(),
                percent: 8.0,
            }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 16948.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_plan_policy_engine(&flags::new_registry());
        let item = new_plan_policy("code-951", "name-360");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
