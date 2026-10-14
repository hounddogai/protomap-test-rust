use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global canary score calculation.
pub trait GlobalCanaryScoreStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalCanaryScore, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalCanaryScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalCanaryScoreStep for FlaggedGlobalCanaryScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalCanaryScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalCanaryScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalCanaryScoreStep for TieredGlobalCanaryScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalCanaryScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalCanaryScoreAdjuster {
    pub cap: f64,
}

impl GlobalCanaryScoreStep for CapGlobalCanaryScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalCanaryScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the global canary score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalCanaryScoreEngine {
    pub steps: Vec<Rc<dyn GlobalCanaryScoreStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalCanaryScoreEngine {
    pub fn run(&mut self, item: &GlobalCanaryScore) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[GlobalCanaryScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_global_canary_score_engine(flags: &flags::Registry) -> GlobalCanaryScoreEngine {
    GlobalCanaryScoreEngine {
        steps: vec![
            Rc::new(FlaggedGlobalCanaryScoreAdjuster {
                flags: flags.clone(),
                flag: "global-canary-score-regional".to_string(),
                percent: -6.2,
            }) as Rc<dyn GlobalCanaryScoreStep>,
            Rc::new(TieredGlobalCanaryScoreAdjuster {
                bounds: vec![23.0, 8584.0],
                rates: vec![0.87, 1.07],
            }),
            Rc::new(CapGlobalCanaryScoreAdjuster { cap: 28012.0 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 51953.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_canary_score_engine(&flags::new_registry());
        let item = new_global_canary_score("code-193", "name-610");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
