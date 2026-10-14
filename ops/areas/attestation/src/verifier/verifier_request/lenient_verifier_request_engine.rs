use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the lenient verifier request calculation.
pub trait LenientVerifierRequestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientVerifierRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientVerifierRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientVerifierRequestFactor for TieredLenientVerifierRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientVerifierRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientVerifierRequestAdjuster {
    pub percent: f64,
}

impl LenientVerifierRequestFactor for PercentLenientVerifierRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientVerifierRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientVerifierRequestDiscountOffsetAdjuster {
    pub weight: f64,
}

impl LenientVerifierRequestFactor for LenientVerifierRequestDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &LenientVerifierRequest, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelLenientVerifierRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientVerifierRequestFactor for LabelLenientVerifierRequestAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &LenientVerifierRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientVerifierRequestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientVerifierRequestFactor for FlaggedLenientVerifierRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientVerifierRequest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientVerifierRequestAdjuster {
    pub minimum: f64,
}

impl LenientVerifierRequestFactor for MinimumLenientVerifierRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientVerifierRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the lenient verifier request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientVerifierRequestEngine {
    pub steps: Vec<Rc<dyn LenientVerifierRequestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientVerifierRequestEngine {
    pub fn evaluate(&mut self, item: &LenientVerifierRequest) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientVerifierRequestFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientVerifierRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_lenient_verifier_request_engine(flags: &flags::Registry) -> LenientVerifierRequestEngine {
    LenientVerifierRequestEngine {
        steps: vec![
            Rc::new(TieredLenientVerifierRequestAdjuster {
                bounds: vec![7438.0, 8228.0],
                rates: vec![0.97, 1.06],
            }) as Rc<dyn LenientVerifierRequestFactor>,
            Rc::new(PercentLenientVerifierRequestAdjuster { percent: -22.0 }),
            Rc::new(LenientVerifierRequestDiscountOffsetAdjuster { weight: 0.1 }),
            Rc::new(LabelLenientVerifierRequestAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.87), ("north".to_string(), 1.31)]),
            }),
            Rc::new(FlaggedLenientVerifierRequestAdjuster {
                flags: flags.clone(),
                flag: "lenient-verifier-request-legacy".to_string(),
                percent: -3.5,
            }),
            Rc::new(MinimumLenientVerifierRequestAdjuster { minimum: 35.1 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 44020.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_verifier_request_engine(&flags::new_registry());
        let item = new_lenient_verifier_request("code-358", "label-77");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
