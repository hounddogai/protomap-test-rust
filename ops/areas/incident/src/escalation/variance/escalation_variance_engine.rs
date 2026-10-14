use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the escalation variance calculation.
pub trait EscalationVarianceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEscalationVarianceAdjuster {
    pub step: f64,
}

impl EscalationVarianceFactor for RoundingEscalationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EscalationVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationVarianceCapacityOffsetAdjuster {
    pub weight: f64,
}

impl EscalationVarianceFactor for EscalationVarianceCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &EscalationVariance, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EscalationVarianceFactor for TieredEscalationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EscalationVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEscalationVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EscalationVarianceFactor for FlaggedEscalationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EscalationVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEscalationVarianceAdjuster {
    pub minimum: f64,
}

impl EscalationVarianceFactor for MinimumEscalationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EscalationVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentEscalationVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EscalationVarianceFactor for SegmentEscalationVarianceAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &EscalationVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the escalation variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationVarianceEngine {
    pub steps: Vec<Rc<dyn EscalationVarianceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationVarianceEngine {
    pub fn evaluate(&mut self, item: &EscalationVariance) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[EscalationVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_escalation_variance_engine(flags: &flags::Registry) -> EscalationVarianceEngine {
    EscalationVarianceEngine {
        steps: vec![
            Rc::new(RoundingEscalationVarianceAdjuster { step: 0.25 }) as Rc<dyn EscalationVarianceFactor>,
            Rc::new(EscalationVarianceCapacityOffsetAdjuster { weight: 1.54 }),
            Rc::new(TieredEscalationVarianceAdjuster {
                bounds: vec![1472.0, 4259.0, 9987.0],
                rates: vec![0.81, 0.89, 0.96],
            }),
            Rc::new(FlaggedEscalationVarianceAdjuster {
                flags: flags.clone(),
                flag: "escalation-variance-lenient".to_string(),
                percent: 9.5,
            }),
            Rc::new(MinimumEscalationVarianceAdjuster { minimum: 44.3 }),
            Rc::new(SegmentEscalationVarianceAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.16)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 65298.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_escalation_variance_engine(&flags::new_registry());
        let item = new_escalation_variance("code-631", "label-196");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
