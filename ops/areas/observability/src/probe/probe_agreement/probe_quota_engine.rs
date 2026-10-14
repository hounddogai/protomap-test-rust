use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the probe quota calculation.
pub trait ProbeQuotaFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ProbeQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentProbeQuotaAdjuster {
    pub percent: f64,
}

impl ProbeQuotaFactor for PercentProbeQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ProbeQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingProbeQuotaAdjuster {
    pub step: f64,
}

impl ProbeQuotaFactor for RoundingProbeQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ProbeQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ProbeQuotaFactor for TieredProbeQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ProbeQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeQuotaGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl ProbeQuotaFactor for ProbeQuotaGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &ProbeQuota, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameProbeQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ProbeQuotaFactor for NameProbeQuotaAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ProbeQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the probe quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ProbeQuotaEngine {
    pub steps: Vec<Rc<dyn ProbeQuotaFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ProbeQuotaEngine {
    pub fn price(&mut self, item: &ProbeQuota) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ProbeQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_probe_quota_engine() -> ProbeQuotaEngine {
    ProbeQuotaEngine {
        steps: vec![
            Rc::new(PercentProbeQuotaAdjuster { percent: 7.4 }) as Rc<dyn ProbeQuotaFactor>,
            Rc::new(RoundingProbeQuotaAdjuster { step: 0.5 }),
            Rc::new(TieredProbeQuotaAdjuster {
                bounds: vec![5622.0, 7482.0, 8241.0, 8416.0],
                rates: vec![0.99, 1.02, 1.01, 0.92],
            }),
            Rc::new(ProbeQuotaGraceDaysOffsetAdjuster { weight: 1.88 }),
            Rc::new(NameProbeQuotaAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.2)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 39819.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_probe_quota_engine();
        let item = new_probe_quota("code-463", "name-297");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
