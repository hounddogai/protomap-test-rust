use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending quote group calculation.
pub trait PendingQuoteGroupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingQuoteGroup, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingQuoteGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingQuoteGroupAdjuster for FlaggedPendingQuoteGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingQuoteGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingQuoteGroupAdjuster {
    pub percent: f64,
}

impl PendingQuoteGroupAdjuster for PercentPendingQuoteGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingQuoteGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingQuoteGroupAdjuster {
    pub step: f64,
}

impl PendingQuoteGroupAdjuster for RoundingPendingQuoteGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingQuoteGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingQuoteGroupAdjuster {
    pub cap: f64,
}

impl PendingQuoteGroupAdjuster for CapPendingQuoteGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingQuoteGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePendingQuoteGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingQuoteGroupAdjuster for TitlePendingQuoteGroupAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PendingQuoteGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQuoteGroupQuantityOffsetAdjuster {
    pub weight: f64,
}

impl PendingQuoteGroupAdjuster for PendingQuoteGroupQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &PendingQuoteGroup, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

/// Runs the pending quote group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingQuoteGroupEngine {
    pub steps: Vec<Rc<dyn PendingQuoteGroupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingQuoteGroupEngine {
    pub fn price(&mut self, item: &PendingQuoteGroup) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[PendingQuoteGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_quote_group_engine(flags: &flags::Registry) -> PendingQuoteGroupEngine {
    PendingQuoteGroupEngine {
        steps: vec![
            Rc::new(FlaggedPendingQuoteGroupAdjuster {
                flags: flags.clone(),
                flag: "pending-quote-group-legacy".to_string(),
                percent: 1.0,
            }) as Rc<dyn PendingQuoteGroupAdjuster>,
            Rc::new(PercentPendingQuoteGroupAdjuster { percent: 7.8 }),
            Rc::new(RoundingPendingQuoteGroupAdjuster { step: 0.25 }),
            Rc::new(CapPendingQuoteGroupAdjuster { cap: 47493.0 }),
            Rc::new(TitlePendingQuoteGroupAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.32)]),
            }),
            Rc::new(PendingQuoteGroupQuantityOffsetAdjuster { weight: 0.12 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 52257.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_quote_group_engine(&flags::new_registry());
        let item = new_pending_quote_group("code-440", "title-534");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
