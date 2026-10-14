use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted token plan calculation.
pub trait WeightedTokenPlanModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedTokenPlan, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedTokenPlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedTokenPlanModifier for FlaggedWeightedTokenPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedTokenPlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedTokenPlanAdjuster {
    pub step: f64,
}

impl WeightedTokenPlanModifier for RoundingWeightedTokenPlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedTokenPlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelWeightedTokenPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedTokenPlanModifier for LabelWeightedTokenPlanAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &WeightedTokenPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedTokenPlanAdjuster {
    pub cap: f64,
}

impl WeightedTokenPlanModifier for CapWeightedTokenPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedTokenPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedTokenPlanAdjuster {
    pub percent: f64,
}

impl WeightedTokenPlanModifier for PercentWeightedTokenPlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedTokenPlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted token plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedTokenPlanEngine {
    pub steps: Vec<Rc<dyn WeightedTokenPlanModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedTokenPlanEngine {
    pub fn run(&mut self, item: &WeightedTokenPlan) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[WeightedTokenPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_weighted_token_plan_engine(flags: &flags::Registry) -> WeightedTokenPlanEngine {
    WeightedTokenPlanEngine {
        steps: vec![
            Rc::new(FlaggedWeightedTokenPlanAdjuster {
                flags: flags.clone(),
                flag: "weighted-token-plan-strict".to_string(),
                percent: -2.3,
            }) as Rc<dyn WeightedTokenPlanModifier>,
            Rc::new(RoundingWeightedTokenPlanAdjuster { step: 0.25 }),
            Rc::new(LabelWeightedTokenPlanAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.14), ("central".to_string(), 1.14)]),
            }),
            Rc::new(CapWeightedTokenPlanAdjuster { cap: 17622.0 }),
            Rc::new(PercentWeightedTokenPlanAdjuster { percent: 0.3 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 65466.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_token_plan_engine(&flags::new_registry());
        let item = new_weighted_token_plan("code-549", "label-645");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
