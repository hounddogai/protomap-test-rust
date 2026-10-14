use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the grant record calculation.
pub trait GrantRecordModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GrantRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGrantRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GrantRecordModifier for TieredGrantRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GrantRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGrantRecordAdjuster {
    pub step: f64,
}

impl GrantRecordModifier for RoundingGrantRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GrantRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantRecordDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl GrantRecordModifier for GrantRecordDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &GrantRecord, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGrantRecordAdjuster {
    pub minimum: f64,
}

impl GrantRecordModifier for MinimumGrantRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GrantRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGrantRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GrantRecordModifier for FlaggedGrantRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GrantRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the grant record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GrantRecordEngine {
    pub steps: Vec<Rc<dyn GrantRecordModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GrantRecordEngine {
    pub fn run(&mut self, item: &GrantRecord) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GrantRecordModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GrantRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_grant_record_engine(flags: &flags::Registry) -> GrantRecordEngine {
    GrantRecordEngine {
        steps: vec![
            Rc::new(TieredGrantRecordAdjuster {
                bounds: vec![874.0, 7912.0, 8102.0, 8429.0],
                rates: vec![0.83, 0.83, 1.0, 0.89],
            }) as Rc<dyn GrantRecordModifier>,
            Rc::new(RoundingGrantRecordAdjuster { step: 0.5 }),
            Rc::new(GrantRecordDurationMinutesOffsetAdjuster { weight: 0.78 }),
            Rc::new(MinimumGrantRecordAdjuster { minimum: 30.8 }),
            Rc::new(FlaggedGrantRecordAdjuster {
                flags: flags.clone(),
                flag: "grant-record-tiered".to_string(),
                percent: 3.7,
            }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 57303.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_grant_record_engine(&flags::new_registry());
        let item = new_grant_record("code-800", "label-765");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
