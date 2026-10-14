use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the federation metric calculation.
pub trait FederationMetricModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeFederationMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FederationMetricModifier for ScopeFederationMetricAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &FederationMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFederationMetricAdjuster {
    pub step: f64,
}

impl FederationMetricModifier for RoundingFederationMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FederationMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFederationMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FederationMetricModifier for FlaggedFederationMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FederationMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FederationMetricModifier for TieredFederationMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FederationMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFederationMetricAdjuster {
    pub cap: f64,
}

impl FederationMetricModifier for CapFederationMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FederationMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the federation metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationMetricEngine {
    pub steps: Vec<Rc<dyn FederationMetricModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationMetricEngine {
    pub fn price(&mut self, item: &FederationMetric) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FederationMetricModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FederationMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_federation_metric_engine(flags: &flags::Registry) -> FederationMetricEngine {
    FederationMetricEngine {
        steps: vec![
            Rc::new(ScopeFederationMetricAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.98)]),
            }) as Rc<dyn FederationMetricModifier>,
            Rc::new(RoundingFederationMetricAdjuster { step: 5.0 }),
            Rc::new(FlaggedFederationMetricAdjuster {
                flags: flags.clone(),
                flag: "federation-metric-legacy".to_string(),
                percent: -3.4,
            }),
            Rc::new(TieredFederationMetricAdjuster {
                bounds: vec![7102.0, 7722.0],
                rates: vec![0.85, 1.13],
            }),
            Rc::new(CapFederationMetricAdjuster { cap: 13525.0 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 27703.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_federation_metric_engine(&flags::new_registry());
        let item = new_federation_metric("code-297", "label-82");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
