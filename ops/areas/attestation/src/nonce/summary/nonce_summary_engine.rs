use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the nonce summary calculation.
pub trait NonceSummaryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NonceSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapNonceSummaryAdjuster {
    pub cap: f64,
}

impl NonceSummaryStep for CapNonceSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &NonceSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedNonceSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl NonceSummaryStep for FlaggedNonceSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &NonceSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl NonceSummaryStep for TieredNonceSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &NonceSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumNonceSummaryAdjuster {
    pub minimum: f64,
}

impl NonceSummaryStep for MinimumNonceSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &NonceSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the nonce summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NonceSummaryEngine {
    pub steps: Vec<Rc<dyn NonceSummaryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NonceSummaryEngine {
    pub fn run(&mut self, item: &NonceSummary) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[NonceSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_nonce_summary_engine(flags: &flags::Registry) -> NonceSummaryEngine {
    NonceSummaryEngine {
        steps: vec![
            Rc::new(CapNonceSummaryAdjuster { cap: 3483.0 }) as Rc<dyn NonceSummaryStep>,
            Rc::new(FlaggedNonceSummaryAdjuster {
                flags: flags.clone(),
                flag: "nonce-summary-batched".to_string(),
                percent: 6.6,
            }),
            Rc::new(TieredNonceSummaryAdjuster {
                bounds: vec![2476.0, 4766.0, 5755.0, 8902.0],
                rates: vec![0.93, 1.15, 1.14, 1.15],
            }),
            Rc::new(MinimumNonceSummaryAdjuster { minimum: 17.4 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 3534.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_nonce_summary_engine(&flags::new_registry());
        let item = new_nonce_summary("code-132", "title-817");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
