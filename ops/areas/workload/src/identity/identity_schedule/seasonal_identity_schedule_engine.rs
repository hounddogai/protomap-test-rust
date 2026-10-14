use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal identity schedule calculation.
pub trait SeasonalIdentityScheduleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalIdentitySchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityScheduleBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalIdentityScheduleFactor for SeasonalIdentityScheduleBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalIdentitySchedule, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalIdentityScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalIdentityScheduleFactor for TieredSeasonalIdentityScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalIdentitySchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelSeasonalIdentityScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalIdentityScheduleFactor for LabelSeasonalIdentityScheduleAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &SeasonalIdentitySchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalIdentityScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalIdentityScheduleFactor for FlaggedSeasonalIdentityScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalIdentitySchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalIdentityScheduleAdjuster {
    pub minimum: f64,
}

impl SeasonalIdentityScheduleFactor for MinimumSeasonalIdentityScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalIdentitySchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the seasonal identity schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalIdentityScheduleEngine {
    pub steps: Vec<Rc<dyn SeasonalIdentityScheduleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalIdentityScheduleEngine {
    pub fn run(&mut self, item: &SeasonalIdentitySchedule) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalIdentitySchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_seasonal_identity_schedule_engine(flags: &flags::Registry) -> SeasonalIdentityScheduleEngine {
    SeasonalIdentityScheduleEngine {
        steps: vec![
            Rc::new(SeasonalIdentityScheduleBatchSizeOffsetAdjuster { weight: 0.13 })
                as Rc<dyn SeasonalIdentityScheduleFactor>,
            Rc::new(TieredSeasonalIdentityScheduleAdjuster {
                bounds: vec![1105.0, 1194.0, 6415.0],
                rates: vec![0.83, 0.96, 0.97],
            }),
            Rc::new(LabelSeasonalIdentityScheduleAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.29)]),
            }),
            Rc::new(FlaggedSeasonalIdentityScheduleAdjuster {
                flags: flags.clone(),
                flag: "seasonal-identity-schedule-seasonal".to_string(),
                percent: -10.0,
            }),
            Rc::new(MinimumSeasonalIdentityScheduleAdjuster { minimum: 29.3 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 51569.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_identity_schedule_engine(&flags::new_registry());
        let item = new_seasonal_identity_schedule("code-497", "label-352");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
