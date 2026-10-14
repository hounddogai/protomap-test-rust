use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the weighted plan window calculation.
pub trait WeightedPlanWindowModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedPlanWindow, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedPlanWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedPlanWindowModifier for FlaggedWeightedPlanWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedPlanWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedPlanWindowAdjuster {
    pub cap: f64,
}

impl WeightedPlanWindowModifier for CapWeightedPlanWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedPlanWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedPlanWindowAdjuster {
    pub percent: f64,
}

impl WeightedPlanWindowModifier for PercentWeightedPlanWindowAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedPlanWindow, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedPlanWindowAdjuster {
    pub minimum: f64,
}

impl WeightedPlanWindowModifier for MinimumWeightedPlanWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedPlanWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPlanWindowCeilingOffsetAdjuster {
    pub weight: f64,
}

impl WeightedPlanWindowModifier for WeightedPlanWindowCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedPlanWindow, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

/// Runs the weighted plan window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedPlanWindowEngine {
    pub steps: Vec<Rc<dyn WeightedPlanWindowModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedPlanWindowEngine {
    pub fn compute(&mut self, item: &WeightedPlanWindow) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedPlanWindowModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedPlanWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_weighted_plan_window_engine(flags: &flags::Registry) -> WeightedPlanWindowEngine {
    WeightedPlanWindowEngine {
        steps: vec![
            Rc::new(FlaggedWeightedPlanWindowAdjuster {
                flags: flags.clone(),
                flag: "weighted-plan-window-tiered".to_string(),
                percent: -8.3,
            }) as Rc<dyn WeightedPlanWindowModifier>,
            Rc::new(CapWeightedPlanWindowAdjuster { cap: 30430.0 }),
            Rc::new(PercentWeightedPlanWindowAdjuster { percent: -4.2 }),
            Rc::new(MinimumWeightedPlanWindowAdjuster { minimum: 40.3 }),
            Rc::new(WeightedPlanWindowCeilingOffsetAdjuster { weight: 0.42 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 58791.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_plan_window_engine(&flags::new_registry());
        let item = new_weighted_plan_window("code-295", "title-336");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
