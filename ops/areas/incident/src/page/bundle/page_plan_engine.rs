use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the page plan calculation.
pub trait PagePlanStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PagePlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPagePlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PagePlanStep for LabelPagePlanAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PagePlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPagePlanAdjuster {
    pub percent: f64,
}

impl PagePlanStep for PercentPagePlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PagePlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PagePlanGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl PagePlanStep for PagePlanGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &PagePlan, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

/// Runs the page plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PagePlanEngine {
    pub steps: Vec<Rc<dyn PagePlanStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PagePlanEngine {
    pub fn price(&mut self, item: &PagePlan) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PagePlanStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PagePlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_page_plan_engine() -> PagePlanEngine {
    PagePlanEngine {
        steps: vec![
            Rc::new(LabelPagePlanAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.75),
                    ("online".to_string(), 1.09),
                    ("west".to_string(), 1.11),
                ]),
            }) as Rc<dyn PagePlanStep>,
            Rc::new(PercentPagePlanAdjuster { percent: 2.8 }),
            Rc::new(PagePlanGraceDaysOffsetAdjuster { weight: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 24811.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_page_plan_engine();
        let item = new_page_plan("code-23", "label-899");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
