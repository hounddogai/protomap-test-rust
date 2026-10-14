use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the probe group calculation.
pub trait ProbeGroupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ProbeGroup, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedProbeGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ProbeGroupModifier for FlaggedProbeGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ProbeGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumProbeGroupAdjuster {
    pub minimum: f64,
}

impl ProbeGroupModifier for MinimumProbeGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ProbeGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ProbeGroupModifier for TieredProbeGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ProbeGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeGroupQuantityOffsetAdjuster {
    pub weight: f64,
}

impl ProbeGroupModifier for ProbeGroupQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &ProbeGroup, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

/// Runs the probe group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ProbeGroupEngine {
    pub steps: Vec<Rc<dyn ProbeGroupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ProbeGroupEngine {
    pub fn compute(&mut self, item: &ProbeGroup) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ProbeGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_probe_group_engine(flags: &flags::Registry) -> ProbeGroupEngine {
    ProbeGroupEngine {
        steps: vec![
            Rc::new(FlaggedProbeGroupAdjuster {
                flags: flags.clone(),
                flag: "probe-group-capped".to_string(),
                percent: 9.2,
            }) as Rc<dyn ProbeGroupModifier>,
            Rc::new(MinimumProbeGroupAdjuster { minimum: 31.6 }),
            Rc::new(TieredProbeGroupAdjuster {
                bounds: vec![2604.0, 7737.0, 8893.0, 9287.0],
                rates: vec![1.01, 1.12, 0.97, 1.0],
            }),
            Rc::new(ProbeGroupQuantityOffsetAdjuster { weight: 0.16 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 90339.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_probe_group_engine(&flags::new_registry());
        let item = new_probe_group("code-452", "label-161");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
