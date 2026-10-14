use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the probe exception calculation.
pub trait ProbeExceptionStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ProbeException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapProbeExceptionAdjuster {
    pub cap: f64,
}

impl ProbeExceptionStep for CapProbeExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ProbeException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumProbeExceptionAdjuster {
    pub minimum: f64,
}

impl ProbeExceptionStep for MinimumProbeExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ProbeException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedProbeExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ProbeExceptionStep for FlaggedProbeExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ProbeException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the probe exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ProbeExceptionEngine {
    pub steps: Vec<Rc<dyn ProbeExceptionStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ProbeExceptionEngine {
    pub fn price(&mut self, item: &ProbeException) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[ProbeException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_probe_exception_engine(flags: &flags::Registry) -> ProbeExceptionEngine {
    ProbeExceptionEngine {
        steps: vec![
            Rc::new(CapProbeExceptionAdjuster { cap: 45429.0 }) as Rc<dyn ProbeExceptionStep>,
            Rc::new(MinimumProbeExceptionAdjuster { minimum: 45.4 }),
            Rc::new(FlaggedProbeExceptionAdjuster {
                flags: flags.clone(),
                flag: "probe-exception-default".to_string(),
                percent: -2.8,
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 18375.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_probe_exception_engine(&flags::new_registry());
        let item = new_probe_exception("code-576", "title-523");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
