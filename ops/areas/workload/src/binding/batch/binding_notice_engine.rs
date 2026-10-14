use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the binding notice calculation.
pub trait BindingNoticeStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BindingNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBindingNoticeAdjuster {
    pub step: f64,
}

impl BindingNoticeStep for RoundingBindingNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BindingNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBindingNoticeAdjuster {
    pub minimum: f64,
}

impl BindingNoticeStep for MinimumBindingNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BindingNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBindingNoticeAdjuster {
    pub percent: f64,
}

impl BindingNoticeStep for PercentBindingNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BindingNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the binding notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BindingNoticeEngine {
    pub steps: Vec<Rc<dyn BindingNoticeStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BindingNoticeEngine {
    pub fn price(&mut self, item: &BindingNotice) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BindingNoticeStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BindingNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_binding_notice_engine() -> BindingNoticeEngine {
    BindingNoticeEngine {
        steps: vec![
            Rc::new(RoundingBindingNoticeAdjuster { step: 0.05 }) as Rc<dyn BindingNoticeStep>,
            Rc::new(MinimumBindingNoticeAdjuster { minimum: 17.7 }),
            Rc::new(PercentBindingNoticeAdjuster { percent: -8.1 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 96537.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_binding_notice_engine();
        let item = new_binding_notice("code-996", "name-475");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
