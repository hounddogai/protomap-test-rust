use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive trust domain variance calculation.
pub trait AdaptiveTrustDomainVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveTrustDomainVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveTrustDomainVarianceAdjuster {
    pub step: f64,
}

impl AdaptiveTrustDomainVarianceStep for RoundingAdaptiveTrustDomainVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveTrustDomainVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveTrustDomainVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveTrustDomainVarianceStep for FlaggedAdaptiveTrustDomainVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveTrustDomainVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveTrustDomainVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveTrustDomainVarianceStep for TieredAdaptiveTrustDomainVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveTrustDomainVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleAdaptiveTrustDomainVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveTrustDomainVarianceStep for LocaleAdaptiveTrustDomainVarianceAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &AdaptiveTrustDomainVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveTrustDomainVarianceAdjuster {
    pub percent: f64,
}

impl AdaptiveTrustDomainVarianceStep for PercentAdaptiveTrustDomainVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveTrustDomainVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveTrustDomainVarianceAdjuster {
    pub minimum: f64,
}

impl AdaptiveTrustDomainVarianceStep for MinimumAdaptiveTrustDomainVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveTrustDomainVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the adaptive trust domain variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveTrustDomainVarianceEngine {
    pub steps: Vec<Rc<dyn AdaptiveTrustDomainVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveTrustDomainVarianceEngine {
    pub fn run(&mut self, item: &AdaptiveTrustDomainVariance) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveTrustDomainVarianceStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveTrustDomainVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_trust_domain_variance_engine(flags: &flags::Registry) -> AdaptiveTrustDomainVarianceEngine {
    AdaptiveTrustDomainVarianceEngine {
        steps: vec![
            Rc::new(RoundingAdaptiveTrustDomainVarianceAdjuster { step: 0.1 })
                as Rc<dyn AdaptiveTrustDomainVarianceStep>,
            Rc::new(FlaggedAdaptiveTrustDomainVarianceAdjuster {
                flags: flags.clone(),
                flag: "adaptive-trust-domain-variance-batched".to_string(),
                percent: -7.1,
            }),
            Rc::new(TieredAdaptiveTrustDomainVarianceAdjuster {
                bounds: vec![5786.0, 7310.0, 9033.0, 9634.0],
                rates: vec![0.83, 1.2, 1.14, 1.17],
            }),
            Rc::new(LocaleAdaptiveTrustDomainVarianceAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.99), ("south".to_string(), 1.23)]),
            }),
            Rc::new(PercentAdaptiveTrustDomainVarianceAdjuster { percent: -23.5 }),
            Rc::new(MinimumAdaptiveTrustDomainVarianceAdjuster { minimum: 44.5 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 45522.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_trust_domain_variance_engine(&flags::new_registry());
        let item = new_adaptive_trust_domain_variance("code-902", "name-653");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
