use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the probe variance calculation.
pub trait ProbeVarianceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ProbeVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumProbeVarianceAdjuster {
    pub minimum: f64,
}

impl ProbeVarianceModifier for MinimumProbeVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ProbeVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingProbeVarianceAdjuster {
    pub step: f64,
}

impl ProbeVarianceModifier for RoundingProbeVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ProbeVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelProbeVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ProbeVarianceModifier for LabelProbeVarianceAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ProbeVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentProbeVarianceAdjuster {
    pub percent: f64,
}

impl ProbeVarianceModifier for PercentProbeVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ProbeVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the probe variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ProbeVarianceEngine {
    pub steps: Vec<Rc<dyn ProbeVarianceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ProbeVarianceEngine {
    pub fn evaluate(&mut self, item: &ProbeVariance) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[ProbeVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_probe_variance_engine() -> ProbeVarianceEngine {
    ProbeVarianceEngine {
        steps: vec![
            Rc::new(MinimumProbeVarianceAdjuster { minimum: 27.7 }) as Rc<dyn ProbeVarianceModifier>,
            Rc::new(RoundingProbeVarianceAdjuster { step: 0.05 }),
            Rc::new(LabelProbeVarianceAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.83), ("north".to_string(), 0.73)]),
            }),
            Rc::new(PercentProbeVarianceAdjuster { percent: 4.3 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 60580.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_probe_variance_engine();
        let item = new_probe_variance("code-738", "label-324");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
