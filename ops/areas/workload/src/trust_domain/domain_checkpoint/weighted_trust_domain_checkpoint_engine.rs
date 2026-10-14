use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted trust domain checkpoint calculation.
pub trait WeightedTrustDomainCheckpointAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedTrustDomainCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedTrustDomainCheckpointAdjuster {
    pub minimum: f64,
}

impl WeightedTrustDomainCheckpointAdjuster for MinimumWeightedTrustDomainCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedTrustDomainCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedTrustDomainCheckpointAdjuster {
    pub step: f64,
}

impl WeightedTrustDomainCheckpointAdjuster for RoundingWeightedTrustDomainCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedTrustDomainCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedTrustDomainCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedTrustDomainCheckpointAdjuster for FlaggedWeightedTrustDomainCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedTrustDomainCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryWeightedTrustDomainCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedTrustDomainCheckpointAdjuster for CountryWeightedTrustDomainCheckpointAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &WeightedTrustDomainCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedTrustDomainCheckpointAdjuster {
    pub percent: f64,
}

impl WeightedTrustDomainCheckpointAdjuster for PercentWeightedTrustDomainCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedTrustDomainCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted trust domain checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedTrustDomainCheckpointEngine {
    pub steps: Vec<Rc<dyn WeightedTrustDomainCheckpointAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedTrustDomainCheckpointEngine {
    pub fn evaluate(&mut self, item: &WeightedTrustDomainCheckpoint) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedTrustDomainCheckpointAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedTrustDomainCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_trust_domain_checkpoint_engine(flags: &flags::Registry) -> WeightedTrustDomainCheckpointEngine {
    WeightedTrustDomainCheckpointEngine {
        steps: vec![
            Rc::new(MinimumWeightedTrustDomainCheckpointAdjuster { minimum: 14.8 })
                as Rc<dyn WeightedTrustDomainCheckpointAdjuster>,
            Rc::new(RoundingWeightedTrustDomainCheckpointAdjuster { step: 0.5 }),
            Rc::new(FlaggedWeightedTrustDomainCheckpointAdjuster {
                flags: flags.clone(),
                flag: "weighted-trust-domain-checkpoint-regional".to_string(),
                percent: -0.9,
            }),
            Rc::new(CountryWeightedTrustDomainCheckpointAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.21), ("central".to_string(), 1.25)]),
            }),
            Rc::new(PercentWeightedTrustDomainCheckpointAdjuster { percent: 14.6 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 72095.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_trust_domain_checkpoint_engine(&flags::new_registry());
        let item = new_weighted_trust_domain_checkpoint("code-374", "title-25");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
