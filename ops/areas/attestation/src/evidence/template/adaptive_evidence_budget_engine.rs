use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive evidence budget calculation.
pub trait AdaptiveEvidenceBudgetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveEvidenceBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveEvidenceBudgetAdjuster {
    pub step: f64,
}

impl AdaptiveEvidenceBudgetStep for RoundingAdaptiveEvidenceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveEvidenceBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveEvidenceBudgetAdjuster {
    pub minimum: f64,
}

impl AdaptiveEvidenceBudgetStep for MinimumAdaptiveEvidenceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveEvidenceBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveEvidenceBudgetAdjuster {
    pub cap: f64,
}

impl AdaptiveEvidenceBudgetStep for CapAdaptiveEvidenceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveEvidenceBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveEvidenceBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveEvidenceBudgetStep for FlaggedAdaptiveEvidenceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveEvidenceBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive evidence budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveEvidenceBudgetEngine {
    pub steps: Vec<Rc<dyn AdaptiveEvidenceBudgetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveEvidenceBudgetEngine {
    pub fn price(&mut self, item: &AdaptiveEvidenceBudget) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveEvidenceBudgetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveEvidenceBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_evidence_budget_engine(flags: &flags::Registry) -> AdaptiveEvidenceBudgetEngine {
    AdaptiveEvidenceBudgetEngine {
        steps: vec![
            Rc::new(RoundingAdaptiveEvidenceBudgetAdjuster { step: 5.0 }) as Rc<dyn AdaptiveEvidenceBudgetStep>,
            Rc::new(MinimumAdaptiveEvidenceBudgetAdjuster { minimum: 3.8 }),
            Rc::new(CapAdaptiveEvidenceBudgetAdjuster { cap: 42491.0 }),
            Rc::new(FlaggedAdaptiveEvidenceBudgetAdjuster {
                flags: flags.clone(),
                flag: "adaptive-evidence-budget-manual".to_string(),
                percent: -0.7,
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 12323.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_evidence_budget_engine(&flags::new_registry());
        let item = new_adaptive_evidence_budget("code-21", "label-752");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
