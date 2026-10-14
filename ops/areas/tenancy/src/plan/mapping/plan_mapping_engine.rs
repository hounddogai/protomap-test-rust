use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the plan mapping calculation.
pub trait PlanMappingStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPlanMappingAdjuster {
    pub cap: f64,
}

impl PlanMappingStep for CapPlanMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PlanMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPlanMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PlanMappingStep for FlaggedPlanMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PlanMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanMappingAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PlanMappingStep for TieredPlanMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PlanMapping, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPlanMappingAdjuster {
    pub minimum: f64,
}

impl PlanMappingStep for MinimumPlanMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PlanMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the plan mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanMappingEngine {
    pub steps: Vec<Rc<dyn PlanMappingStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanMappingEngine {
    pub fn evaluate(&mut self, item: &PlanMapping) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[PlanMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_plan_mapping_engine(flags: &flags::Registry) -> PlanMappingEngine {
    PlanMappingEngine {
        steps: vec![
            Rc::new(CapPlanMappingAdjuster { cap: 34589.0 }) as Rc<dyn PlanMappingStep>,
            Rc::new(FlaggedPlanMappingAdjuster {
                flags: flags.clone(),
                flag: "plan-mapping-batched".to_string(),
                percent: 3.8,
            }),
            Rc::new(TieredPlanMappingAdjuster {
                bounds: vec![1705.0, 4390.0],
                rates: vec![1.08, 1.14],
            }),
            Rc::new(MinimumPlanMappingAdjuster { minimum: 14.9 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 95466.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_plan_mapping_engine(&flags::new_registry());
        let item = new_plan_mapping("code-589", "name-418");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
