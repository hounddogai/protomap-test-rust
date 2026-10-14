use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the verifier estimate calculation.
pub trait VerifierEstimateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &VerifierEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameVerifierEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl VerifierEstimateModifier for NameVerifierEstimateAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &VerifierEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl VerifierEstimateModifier for TieredVerifierEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &VerifierEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapVerifierEstimateAdjuster {
    pub cap: f64,
}

impl VerifierEstimateModifier for CapVerifierEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &VerifierEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedVerifierEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl VerifierEstimateModifier for FlaggedVerifierEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &VerifierEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingVerifierEstimateAdjuster {
    pub step: f64,
}

impl VerifierEstimateModifier for RoundingVerifierEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &VerifierEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the verifier estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct VerifierEstimateEngine {
    pub steps: Vec<Rc<dyn VerifierEstimateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl VerifierEstimateEngine {
    pub fn run(&mut self, item: &VerifierEstimate) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[VerifierEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_verifier_estimate_engine(flags: &flags::Registry) -> VerifierEstimateEngine {
    VerifierEstimateEngine {
        steps: vec![
            Rc::new(NameVerifierEstimateAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.04),
                    ("east".to_string(), 0.8),
                    ("north".to_string(), 0.96),
                ]),
            }) as Rc<dyn VerifierEstimateModifier>,
            Rc::new(TieredVerifierEstimateAdjuster {
                bounds: vec![4460.0, 8349.0],
                rates: vec![0.91, 1.04],
            }),
            Rc::new(CapVerifierEstimateAdjuster { cap: 46615.0 }),
            Rc::new(FlaggedVerifierEstimateAdjuster {
                flags: flags.clone(),
                flag: "verifier-estimate-tiered".to_string(),
                percent: 6.3,
            }),
            Rc::new(RoundingVerifierEstimateAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 71424.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_verifier_estimate_engine(&flags::new_registry());
        let item = new_verifier_estimate("code-609", "name-576");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
