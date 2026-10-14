use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the page baseline calculation.
pub trait PageBaselineModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PageBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPageBaselineAdjuster {
    pub step: f64,
}

impl PageBaselineModifier for RoundingPageBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PageBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPageBaselineAdjuster {
    pub percent: f64,
}

impl PageBaselineModifier for PercentPageBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PageBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBaselineWeightOffsetAdjuster {
    pub weight: f64,
}

impl PageBaselineModifier for PageBaselineWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &PageBaseline, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPageBaselineAdjuster {
    pub minimum: f64,
}

impl PageBaselineModifier for MinimumPageBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PageBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPageBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PageBaselineModifier for FlaggedPageBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PageBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the page baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PageBaselineEngine {
    pub steps: Vec<Rc<dyn PageBaselineModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PageBaselineEngine {
    pub fn price(&mut self, item: &PageBaseline) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PageBaselineModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PageBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_page_baseline_engine(flags: &flags::Registry) -> PageBaselineEngine {
    PageBaselineEngine {
        steps: vec![
            Rc::new(RoundingPageBaselineAdjuster { step: 0.25 }) as Rc<dyn PageBaselineModifier>,
            Rc::new(PercentPageBaselineAdjuster { percent: -18.8 }),
            Rc::new(PageBaselineWeightOffsetAdjuster { weight: 1.9 }),
            Rc::new(MinimumPageBaselineAdjuster { minimum: 33.2 }),
            Rc::new(FlaggedPageBaselineAdjuster {
                flags: flags.clone(),
                flag: "page-baseline-rolling".to_string(),
                percent: 8.4,
            }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 21147.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_page_baseline_engine(&flags::new_registry());
        let item = new_page_baseline("code-246", "title-639");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
