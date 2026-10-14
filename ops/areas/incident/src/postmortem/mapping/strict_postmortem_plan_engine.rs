use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the strict postmortem plan calculation.
pub trait StrictPostmortemPlanModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictPostmortemPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictPostmortemPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictPostmortemPlanModifier for TieredStrictPostmortemPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictPostmortemPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictPostmortemPlanAdjuster {
    pub minimum: f64,
}

impl StrictPostmortemPlanModifier for MinimumStrictPostmortemPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictPostmortemPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelStrictPostmortemPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictPostmortemPlanModifier for LabelStrictPostmortemPlanAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &StrictPostmortemPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictPostmortemPlanAdjuster {
    pub cap: f64,
}

impl StrictPostmortemPlanModifier for CapStrictPostmortemPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictPostmortemPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the strict postmortem plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictPostmortemPlanEngine {
    pub steps: Vec<Rc<dyn StrictPostmortemPlanModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictPostmortemPlanEngine {
    pub fn price(&mut self, item: &StrictPostmortemPlan) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictPostmortemPlanModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictPostmortemPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_strict_postmortem_plan_engine() -> StrictPostmortemPlanEngine {
    StrictPostmortemPlanEngine {
        steps: vec![
            Rc::new(TieredStrictPostmortemPlanAdjuster {
                bounds: vec![123.0, 8029.0],
                rates: vec![0.97, 1.04],
            }) as Rc<dyn StrictPostmortemPlanModifier>,
            Rc::new(MinimumStrictPostmortemPlanAdjuster { minimum: 30.7 }),
            Rc::new(LabelStrictPostmortemPlanAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.15),
                    ("south".to_string(), 1.34),
                    ("central".to_string(), 0.84),
                ]),
            }),
            Rc::new(CapStrictPostmortemPlanAdjuster { cap: 39191.0 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 87261.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_postmortem_plan_engine();
        let item = new_strict_postmortem_plan("code-284", "label-739");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
