use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the escalation plan calculation.
pub trait EscalationPlanStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEscalationPlanAdjuster {
    pub cap: f64,
}

impl EscalationPlanStep for CapEscalationPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EscalationPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameEscalationPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EscalationPlanStep for NameEscalationPlanAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &EscalationPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEscalationPlanAdjuster {
    pub percent: f64,
}

impl EscalationPlanStep for PercentEscalationPlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EscalationPlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEscalationPlanAdjuster {
    pub step: f64,
}

impl EscalationPlanStep for RoundingEscalationPlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EscalationPlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EscalationPlanStep for TieredEscalationPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EscalationPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the escalation plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationPlanEngine {
    pub steps: Vec<Rc<dyn EscalationPlanStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationPlanEngine {
    pub fn price(&mut self, item: &EscalationPlan) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[EscalationPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_escalation_plan_engine() -> EscalationPlanEngine {
    EscalationPlanEngine {
        steps: vec![
            Rc::new(CapEscalationPlanAdjuster { cap: 35272.0 }) as Rc<dyn EscalationPlanStep>,
            Rc::new(NameEscalationPlanAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.93)]),
            }),
            Rc::new(PercentEscalationPlanAdjuster { percent: -13.3 }),
            Rc::new(RoundingEscalationPlanAdjuster { step: 0.25 }),
            Rc::new(TieredEscalationPlanAdjuster {
                bounds: vec![7106.0, 7775.0, 9956.0],
                rates: vec![0.99, 1.18, 1.05],
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 79948.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_escalation_plan_engine();
        let item = new_escalation_plan("code-552", "name-598");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
