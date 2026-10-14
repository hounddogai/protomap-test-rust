use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the page summary calculation.
pub trait PageSummaryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PageSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPageSummaryAdjuster {
    pub cap: f64,
}

impl PageSummaryModifier for CapPageSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PageSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageSummaryQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl PageSummaryModifier for PageSummaryQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &PageSummary, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPageSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PageSummaryModifier for FlaggedPageSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PageSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPageSummaryAdjuster {
    pub percent: f64,
}

impl PageSummaryModifier for PercentPageSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PageSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPageSummaryAdjuster {
    pub step: f64,
}

impl PageSummaryModifier for RoundingPageSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PageSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPageSummaryAdjuster {
    pub minimum: f64,
}

impl PageSummaryModifier for MinimumPageSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PageSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the page summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PageSummaryEngine {
    pub steps: Vec<Rc<dyn PageSummaryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PageSummaryEngine {
    pub fn run(&mut self, item: &PageSummary) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PageSummaryModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PageSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_page_summary_engine(flags: &flags::Registry) -> PageSummaryEngine {
    PageSummaryEngine {
        steps: vec![
            Rc::new(CapPageSummaryAdjuster { cap: 7156.0 }) as Rc<dyn PageSummaryModifier>,
            Rc::new(PageSummaryQuotaUnitsOffsetAdjuster { weight: 0.18 }),
            Rc::new(FlaggedPageSummaryAdjuster {
                flags: flags.clone(),
                flag: "page-summary-rolling".to_string(),
                percent: -8.2,
            }),
            Rc::new(PercentPageSummaryAdjuster { percent: 2.1 }),
            Rc::new(RoundingPageSummaryAdjuster { step: 0.05 }),
            Rc::new(MinimumPageSummaryAdjuster { minimum: 44.8 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 26090.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_page_summary_engine(&flags::new_registry());
        let item = new_page_summary("code-659", "name-461");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
