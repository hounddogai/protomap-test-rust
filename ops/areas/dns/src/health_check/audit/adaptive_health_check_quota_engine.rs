use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive health check quota calculation.
pub trait AdaptiveHealthCheckQuotaFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveHealthCheckQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveHealthCheckQuotaAdjuster {
    pub cap: f64,
}

impl AdaptiveHealthCheckQuotaFactor for CapAdaptiveHealthCheckQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveHealthCheckQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveHealthCheckQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveHealthCheckQuotaFactor for TieredAdaptiveHealthCheckQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveHealthCheckQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveHealthCheckQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveHealthCheckQuotaFactor for FlaggedAdaptiveHealthCheckQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveHealthCheckQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveHealthCheckQuotaAdjuster {
    pub step: f64,
}

impl AdaptiveHealthCheckQuotaFactor for RoundingAdaptiveHealthCheckQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveHealthCheckQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveHealthCheckQuotaAdjuster {
    pub percent: f64,
}

impl AdaptiveHealthCheckQuotaFactor for PercentAdaptiveHealthCheckQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveHealthCheckQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive health check quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveHealthCheckQuotaEngine {
    pub steps: Vec<Rc<dyn AdaptiveHealthCheckQuotaFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveHealthCheckQuotaEngine {
    pub fn evaluate(&mut self, item: &AdaptiveHealthCheckQuota) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveHealthCheckQuotaFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveHealthCheckQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_health_check_quota_engine(flags: &flags::Registry) -> AdaptiveHealthCheckQuotaEngine {
    AdaptiveHealthCheckQuotaEngine {
        steps: vec![
            Rc::new(CapAdaptiveHealthCheckQuotaAdjuster { cap: 34048.0 }) as Rc<dyn AdaptiveHealthCheckQuotaFactor>,
            Rc::new(TieredAdaptiveHealthCheckQuotaAdjuster {
                bounds: vec![2907.0, 7771.0, 8422.0, 9137.0],
                rates: vec![1.05, 1.18, 1.1, 0.89],
            }),
            Rc::new(FlaggedAdaptiveHealthCheckQuotaAdjuster {
                flags: flags.clone(),
                flag: "adaptive-health-check-quota-tiered".to_string(),
                percent: -4.4,
            }),
            Rc::new(RoundingAdaptiveHealthCheckQuotaAdjuster { step: 0.1 }),
            Rc::new(PercentAdaptiveHealthCheckQuotaAdjuster { percent: 11.8 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 19919.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_health_check_quota_engine(&flags::new_registry());
        let item = new_adaptive_health_check_quota("code-610", "title-156");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
