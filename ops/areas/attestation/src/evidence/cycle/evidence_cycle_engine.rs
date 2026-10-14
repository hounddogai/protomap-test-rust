use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the evidence cycle calculation.
pub trait EvidenceCycleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceCycleScoreOffsetAdjuster {
    pub weight: f64,
}

impl EvidenceCycleAdjuster for EvidenceCycleScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &EvidenceCycle, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEvidenceCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EvidenceCycleAdjuster for FlaggedEvidenceCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EvidenceCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEvidenceCycleAdjuster {
    pub minimum: f64,
}

impl EvidenceCycleAdjuster for MinimumEvidenceCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EvidenceCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEvidenceCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EvidenceCycleAdjuster for TieredEvidenceCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EvidenceCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEvidenceCycleAdjuster {
    pub cap: f64,
}

impl EvidenceCycleAdjuster for CapEvidenceCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EvidenceCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEvidenceCycleAdjuster {
    pub percent: f64,
}

impl EvidenceCycleAdjuster for PercentEvidenceCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EvidenceCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the evidence cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceCycleEngine {
    pub steps: Vec<Rc<dyn EvidenceCycleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceCycleEngine {
    pub fn compute(&mut self, item: &EvidenceCycle) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_evidence_cycle_engine(flags: &flags::Registry) -> EvidenceCycleEngine {
    EvidenceCycleEngine {
        steps: vec![
            Rc::new(EvidenceCycleScoreOffsetAdjuster { weight: 0.9 }) as Rc<dyn EvidenceCycleAdjuster>,
            Rc::new(FlaggedEvidenceCycleAdjuster {
                flags: flags.clone(),
                flag: "evidence-cycle-legacy".to_string(),
                percent: 3.9,
            }),
            Rc::new(MinimumEvidenceCycleAdjuster { minimum: 49.5 }),
            Rc::new(TieredEvidenceCycleAdjuster {
                bounds: vec![4531.0, 8754.0],
                rates: vec![0.82, 0.93],
            }),
            Rc::new(CapEvidenceCycleAdjuster { cap: 28600.0 }),
            Rc::new(PercentEvidenceCycleAdjuster { percent: 21.6 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 51292.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_evidence_cycle_engine(&flags::new_registry());
        let item = new_evidence_cycle("code-611", "name-85");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
