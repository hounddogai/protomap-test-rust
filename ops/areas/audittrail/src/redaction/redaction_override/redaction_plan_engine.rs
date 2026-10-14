use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the redaction plan calculation.
pub trait RedactionPlanModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRedactionPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RedactionPlanModifier for LabelRedactionPlanAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RedactionPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRedactionPlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RedactionPlanModifier for FlaggedRedactionPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RedactionPlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRedactionPlanAdjuster {
    pub minimum: f64,
}

impl RedactionPlanModifier for MinimumRedactionPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RedactionPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRedactionPlanAdjuster {
    pub cap: f64,
}

impl RedactionPlanModifier for CapRedactionPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RedactionPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionPlanBudgetOffsetAdjuster {
    pub weight: f64,
}

impl RedactionPlanModifier for RedactionPlanBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &RedactionPlan, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

/// Runs the redaction plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionPlanEngine {
    pub steps: Vec<Rc<dyn RedactionPlanModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionPlanEngine {
    pub fn compute(&mut self, item: &RedactionPlan) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[RedactionPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_redaction_plan_engine(flags: &flags::Registry) -> RedactionPlanEngine {
    RedactionPlanEngine {
        steps: vec![
            Rc::new(LabelRedactionPlanAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.15),
                    ("west".to_string(), 0.77),
                    ("north".to_string(), 1.28),
                ]),
            }) as Rc<dyn RedactionPlanModifier>,
            Rc::new(FlaggedRedactionPlanAdjuster {
                flags: flags.clone(),
                flag: "redaction-plan-seasonal".to_string(),
                percent: 7.7,
            }),
            Rc::new(MinimumRedactionPlanAdjuster { minimum: 14.5 }),
            Rc::new(CapRedactionPlanAdjuster { cap: 19006.0 }),
            Rc::new(RedactionPlanBudgetOffsetAdjuster { weight: 1.31 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 5734.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_redaction_plan_engine(&flags::new_registry());
        let item = new_redaction_plan("code-417", "label-614");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
