use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the probe metric calculation.
pub trait ProbeMetricStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ProbeMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapProbeMetricAdjuster {
    pub cap: f64,
}

impl ProbeMetricStep for CapProbeMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ProbeMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedProbeMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ProbeMetricStep for FlaggedProbeMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ProbeMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentProbeMetricAdjuster {
    pub percent: f64,
}

impl ProbeMetricStep for PercentProbeMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ProbeMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeMetricCostOffsetAdjuster {
    pub weight: f64,
}

impl ProbeMetricStep for ProbeMetricCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &ProbeMetric, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ProbeMetricStep for TieredProbeMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ProbeMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingProbeMetricAdjuster {
    pub step: f64,
}

impl ProbeMetricStep for RoundingProbeMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ProbeMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the probe metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ProbeMetricEngine {
    pub steps: Vec<Rc<dyn ProbeMetricStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ProbeMetricEngine {
    pub fn price(&mut self, item: &ProbeMetric) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ProbeMetricStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ProbeMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_probe_metric_engine(flags: &flags::Registry) -> ProbeMetricEngine {
    ProbeMetricEngine {
        steps: vec![
            Rc::new(CapProbeMetricAdjuster { cap: 32003.0 }) as Rc<dyn ProbeMetricStep>,
            Rc::new(FlaggedProbeMetricAdjuster {
                flags: flags.clone(),
                flag: "probe-metric-default".to_string(),
                percent: 0.1,
            }),
            Rc::new(PercentProbeMetricAdjuster { percent: -7.5 }),
            Rc::new(ProbeMetricCostOffsetAdjuster { weight: 0.12 }),
            Rc::new(TieredProbeMetricAdjuster {
                bounds: vec![2084.0, 2152.0, 8191.0],
                rates: vec![0.93, 1.01, 0.8],
            }),
            Rc::new(RoundingProbeMetricAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 2336.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_probe_metric_engine(&flags::new_registry());
        let item = new_probe_metric("code-104", "title-935");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
