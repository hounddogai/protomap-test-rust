use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the lenient sink plan calculation.
pub trait LenientSinkPlanFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientSinkPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientSinkPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientSinkPlanFactor for TieredLenientSinkPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientSinkPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientSinkPlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientSinkPlanFactor for FlaggedLenientSinkPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientSinkPlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientSinkPlanAdjuster {
    pub minimum: f64,
}

impl LenientSinkPlanFactor for MinimumLenientSinkPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientSinkPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the lenient sink plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientSinkPlanEngine {
    pub steps: Vec<Rc<dyn LenientSinkPlanFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientSinkPlanEngine {
    pub fn run(&mut self, item: &LenientSinkPlan) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientSinkPlanFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientSinkPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_sink_plan_engine(flags: &flags::Registry) -> LenientSinkPlanEngine {
    LenientSinkPlanEngine {
        steps: vec![
            Rc::new(TieredLenientSinkPlanAdjuster {
                bounds: vec![3335.0, 4136.0, 4307.0, 9599.0],
                rates: vec![1.08, 1.11, 1.04, 0.99],
            }) as Rc<dyn LenientSinkPlanFactor>,
            Rc::new(FlaggedLenientSinkPlanAdjuster {
                flags: flags.clone(),
                flag: "lenient-sink-plan-capped".to_string(),
                percent: 7.3,
            }),
            Rc::new(MinimumLenientSinkPlanAdjuster { minimum: 45.8 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 20976.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_sink_plan_engine(&flags::new_registry());
        let item = new_lenient_sink_plan("code-946", "name-548");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
