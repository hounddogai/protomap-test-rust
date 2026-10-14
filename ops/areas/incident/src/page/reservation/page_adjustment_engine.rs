use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the page adjustment calculation.
pub trait PageAdjustmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PageAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageAdjustmentCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl PageAdjustmentStep for PageAdjustmentCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &PageAdjustment, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPageAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PageAdjustmentStep for TieredPageAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PageAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPageAdjustmentAdjuster {
    pub cap: f64,
}

impl PageAdjustmentStep for CapPageAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PageAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPageAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PageAdjustmentStep for LabelPageAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PageAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPageAdjustmentAdjuster {
    pub minimum: f64,
}

impl PageAdjustmentStep for MinimumPageAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PageAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPageAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PageAdjustmentStep for FlaggedPageAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PageAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the page adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PageAdjustmentEngine {
    pub steps: Vec<Rc<dyn PageAdjustmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PageAdjustmentEngine {
    pub fn price(&mut self, item: &PageAdjustment) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[PageAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_page_adjustment_engine(flags: &flags::Registry) -> PageAdjustmentEngine {
    PageAdjustmentEngine {
        steps: vec![
            Rc::new(PageAdjustmentCreatedAtOffsetAdjuster { weight: 1.1 }) as Rc<dyn PageAdjustmentStep>,
            Rc::new(TieredPageAdjustmentAdjuster {
                bounds: vec![1016.0, 1356.0, 3573.0],
                rates: vec![1.1, 0.86, 1.13],
            }),
            Rc::new(CapPageAdjustmentAdjuster { cap: 5235.0 }),
            Rc::new(LabelPageAdjustmentAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.79)]),
            }),
            Rc::new(MinimumPageAdjustmentAdjuster { minimum: 17.1 }),
            Rc::new(FlaggedPageAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "page-adjustment-legacy".to_string(),
                percent: 5.5,
            }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 42146.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_page_adjustment_engine(&flags::new_registry());
        let item = new_page_adjustment("code-880", "label-410");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
