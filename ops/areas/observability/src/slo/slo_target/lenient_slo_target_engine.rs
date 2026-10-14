use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the lenient slo target calculation.
pub trait LenientSloTargetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientSloTarget, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientSloTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientSloTargetStep for FlaggedLenientSloTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientSloTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientSloTargetAdjuster {
    pub percent: f64,
}

impl LenientSloTargetStep for PercentLenientSloTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientSloTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientSloTargetAdjuster {
    pub cap: f64,
}

impl LenientSloTargetStep for CapLenientSloTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientSloTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloTargetLimitOffsetAdjuster {
    pub weight: f64,
}

impl LenientSloTargetStep for LenientSloTargetLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &LenientSloTarget, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientSloTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientSloTargetStep for TieredLenientSloTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientSloTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientSloTargetAdjuster {
    pub minimum: f64,
}

impl LenientSloTargetStep for MinimumLenientSloTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientSloTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the lenient slo target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientSloTargetEngine {
    pub steps: Vec<Rc<dyn LenientSloTargetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientSloTargetEngine {
    pub fn run(&mut self, item: &LenientSloTarget) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientSloTargetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientSloTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_slo_target_engine(flags: &flags::Registry) -> LenientSloTargetEngine {
    LenientSloTargetEngine {
        steps: vec![
            Rc::new(FlaggedLenientSloTargetAdjuster {
                flags: flags.clone(),
                flag: "lenient-slo-target-default".to_string(),
                percent: -1.9,
            }) as Rc<dyn LenientSloTargetStep>,
            Rc::new(PercentLenientSloTargetAdjuster { percent: 17.1 }),
            Rc::new(CapLenientSloTargetAdjuster { cap: 9380.0 }),
            Rc::new(LenientSloTargetLimitOffsetAdjuster { weight: 0.32 }),
            Rc::new(TieredLenientSloTargetAdjuster {
                bounds: vec![227.0, 3017.0, 6085.0, 7277.0],
                rates: vec![1.18, 1.09, 1.18, 1.16],
            }),
            Rc::new(MinimumLenientSloTargetAdjuster { minimum: 8.2 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 6069.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_slo_target_engine(&flags::new_registry());
        let item = new_lenient_slo_target("code-864", "title-670");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
