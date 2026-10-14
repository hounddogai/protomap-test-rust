use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient overage plan calculation.
pub trait LenientOveragePlanModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientOveragePlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientOveragePlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientOveragePlanModifier for TieredLenientOveragePlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientOveragePlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientOveragePlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientOveragePlanModifier for FlaggedLenientOveragePlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientOveragePlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientOveragePlanAdjuster {
    pub step: f64,
}

impl LenientOveragePlanModifier for RoundingLenientOveragePlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientOveragePlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientOveragePlanAdjuster {
    pub minimum: f64,
}

impl LenientOveragePlanModifier for MinimumLenientOveragePlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientOveragePlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientOveragePlanAdjuster {
    pub percent: f64,
}

impl LenientOveragePlanModifier for PercentLenientOveragePlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientOveragePlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientOveragePlanSequenceOffsetAdjuster {
    pub weight: f64,
}

impl LenientOveragePlanModifier for LenientOveragePlanSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &LenientOveragePlan, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

/// Runs the lenient overage plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientOveragePlanEngine {
    pub steps: Vec<Rc<dyn LenientOveragePlanModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientOveragePlanEngine {
    pub fn evaluate(&mut self, item: &LenientOveragePlan) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientOveragePlanModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientOveragePlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_lenient_overage_plan_engine(flags: &flags::Registry) -> LenientOveragePlanEngine {
    LenientOveragePlanEngine {
        steps: vec![
            Rc::new(TieredLenientOveragePlanAdjuster {
                bounds: vec![1314.0, 2829.0, 5530.0],
                rates: vec![0.85, 0.86, 1.08],
            }) as Rc<dyn LenientOveragePlanModifier>,
            Rc::new(FlaggedLenientOveragePlanAdjuster {
                flags: flags.clone(),
                flag: "lenient-overage-plan-primary".to_string(),
                percent: -5.5,
            }),
            Rc::new(RoundingLenientOveragePlanAdjuster { step: 0.1 }),
            Rc::new(MinimumLenientOveragePlanAdjuster { minimum: 22.2 }),
            Rc::new(PercentLenientOveragePlanAdjuster { percent: -16.3 }),
            Rc::new(LenientOveragePlanSequenceOffsetAdjuster { weight: 0.46 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 21637.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_overage_plan_engine(&flags::new_registry());
        let item = new_lenient_overage_plan("code-173", "title-675");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
