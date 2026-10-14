use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive slo record calculation.
pub trait AdaptiveSloRecordFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveSloRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveSloRecordAdjuster {
    pub cap: f64,
}

impl AdaptiveSloRecordFactor for CapAdaptiveSloRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveSloRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelAdaptiveSloRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveSloRecordFactor for ChannelAdaptiveSloRecordAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &AdaptiveSloRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveSloRecordAdjuster {
    pub percent: f64,
}

impl AdaptiveSloRecordFactor for PercentAdaptiveSloRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveSloRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveSloRecordAdjuster {
    pub minimum: f64,
}

impl AdaptiveSloRecordFactor for MinimumAdaptiveSloRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveSloRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveSloRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveSloRecordFactor for TieredAdaptiveSloRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveSloRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveSloRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveSloRecordFactor for FlaggedAdaptiveSloRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveSloRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive slo record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveSloRecordEngine {
    pub steps: Vec<Rc<dyn AdaptiveSloRecordFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveSloRecordEngine {
    pub fn run(&mut self, item: &AdaptiveSloRecord) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveSloRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_slo_record_engine(flags: &flags::Registry) -> AdaptiveSloRecordEngine {
    AdaptiveSloRecordEngine {
        steps: vec![
            Rc::new(CapAdaptiveSloRecordAdjuster { cap: 12485.0 }) as Rc<dyn AdaptiveSloRecordFactor>,
            Rc::new(ChannelAdaptiveSloRecordAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.89),
                    ("online".to_string(), 1.24),
                    ("east".to_string(), 1.11),
                ]),
            }),
            Rc::new(PercentAdaptiveSloRecordAdjuster { percent: -8.0 }),
            Rc::new(MinimumAdaptiveSloRecordAdjuster { minimum: 46.4 }),
            Rc::new(TieredAdaptiveSloRecordAdjuster {
                bounds: vec![519.0, 739.0, 4347.0, 6703.0],
                rates: vec![1.09, 1.16, 1.08, 1.16],
            }),
            Rc::new(FlaggedAdaptiveSloRecordAdjuster {
                flags: flags.clone(),
                flag: "adaptive-slo-record-tiered".to_string(),
                percent: 5.7,
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 86505.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_slo_record_engine(&flags::new_registry());
        let item = new_adaptive_slo_record("code-954", "name-229");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
