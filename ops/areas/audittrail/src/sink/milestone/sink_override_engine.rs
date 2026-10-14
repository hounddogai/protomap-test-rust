use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the sink override calculation.
pub trait SinkOverrideFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SinkOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSinkOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SinkOverrideFactor for TieredSinkOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SinkOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSinkOverrideAdjuster {
    pub step: f64,
}

impl SinkOverrideFactor for RoundingSinkOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SinkOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameSinkOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SinkOverrideFactor for NameSinkOverrideAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &SinkOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSinkOverrideAdjuster {
    pub minimum: f64,
}

impl SinkOverrideFactor for MinimumSinkOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SinkOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSinkOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SinkOverrideFactor for FlaggedSinkOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SinkOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the sink override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SinkOverrideEngine {
    pub steps: Vec<Rc<dyn SinkOverrideFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SinkOverrideEngine {
    pub fn price(&mut self, item: &SinkOverride) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SinkOverrideFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SinkOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_sink_override_engine(flags: &flags::Registry) -> SinkOverrideEngine {
    SinkOverrideEngine {
        steps: vec![
            Rc::new(TieredSinkOverrideAdjuster {
                bounds: vec![1192.0, 3482.0, 4201.0],
                rates: vec![1.03, 1.16, 1.18],
            }) as Rc<dyn SinkOverrideFactor>,
            Rc::new(RoundingSinkOverrideAdjuster { step: 1.0 }),
            Rc::new(NameSinkOverrideAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.18),
                    ("east".to_string(), 1.08),
                    ("north".to_string(), 0.92),
                ]),
            }),
            Rc::new(MinimumSinkOverrideAdjuster { minimum: 39.3 }),
            Rc::new(FlaggedSinkOverrideAdjuster {
                flags: flags.clone(),
                flag: "sink-override-scheduled".to_string(),
                percent: 9.3,
            }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 61713.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_sink_override_engine(&flags::new_registry());
        let item = new_sink_override("code-415", "name-979");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
