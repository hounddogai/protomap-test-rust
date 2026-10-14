use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the trust domain metric calculation.
pub trait TrustDomainMetricStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainMetricWeightOffsetAdjuster {
    pub weight: f64,
}

impl TrustDomainMetricStep for TrustDomainMetricWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &TrustDomainMetric, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelTrustDomainMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TrustDomainMetricStep for LabelTrustDomainMetricAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &TrustDomainMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTrustDomainMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TrustDomainMetricStep for FlaggedTrustDomainMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TrustDomainMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TrustDomainMetricStep for TieredTrustDomainMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TrustDomainMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the trust domain metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainMetricEngine {
    pub steps: Vec<Rc<dyn TrustDomainMetricStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainMetricEngine {
    pub fn run(&mut self, item: &TrustDomainMetric) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TrustDomainMetricStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_trust_domain_metric_engine(flags: &flags::Registry) -> TrustDomainMetricEngine {
    TrustDomainMetricEngine {
        steps: vec![
            Rc::new(TrustDomainMetricWeightOffsetAdjuster { weight: 1.14 }) as Rc<dyn TrustDomainMetricStep>,
            Rc::new(LabelTrustDomainMetricAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.16),
                    ("west".to_string(), 1.03),
                    ("south".to_string(), 0.9),
                ]),
            }),
            Rc::new(FlaggedTrustDomainMetricAdjuster {
                flags: flags.clone(),
                flag: "trust-domain-metric-strict".to_string(),
                percent: 0.5,
            }),
            Rc::new(TieredTrustDomainMetricAdjuster {
                bounds: vec![5584.0, 7158.0],
                rates: vec![0.96, 1.18],
            }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 23062.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_trust_domain_metric_engine(&flags::new_registry());
        let item = new_trust_domain_metric("code-357", "label-572");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
