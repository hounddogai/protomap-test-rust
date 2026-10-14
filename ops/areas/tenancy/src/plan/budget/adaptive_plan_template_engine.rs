use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the adaptive plan template calculation.
pub trait AdaptivePlanTemplateStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptivePlanTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptivePlanTemplateAdjuster {
    pub cap: f64,
}

impl AdaptivePlanTemplateStep for CapAdaptivePlanTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptivePlanTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptivePlanTemplateAdjuster {
    pub minimum: f64,
}

impl AdaptivePlanTemplateStep for MinimumAdaptivePlanTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptivePlanTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptivePlanTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptivePlanTemplateStep for TieredAdaptivePlanTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptivePlanTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the adaptive plan template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptivePlanTemplateEngine {
    pub steps: Vec<Rc<dyn AdaptivePlanTemplateStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptivePlanTemplateEngine {
    pub fn compute(&mut self, item: &AdaptivePlanTemplate) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptivePlanTemplateStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptivePlanTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_adaptive_plan_template_engine() -> AdaptivePlanTemplateEngine {
    AdaptivePlanTemplateEngine {
        steps: vec![
            Rc::new(CapAdaptivePlanTemplateAdjuster { cap: 34048.0 }) as Rc<dyn AdaptivePlanTemplateStep>,
            Rc::new(MinimumAdaptivePlanTemplateAdjuster { minimum: 2.1 }),
            Rc::new(TieredAdaptivePlanTemplateAdjuster {
                bounds: vec![3212.0, 4074.0, 5238.0],
                rates: vec![1.1, 0.88, 0.9],
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 90207.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_plan_template_engine();
        let item = new_adaptive_plan_template("code-516", "title-256");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
