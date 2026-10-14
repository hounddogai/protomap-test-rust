use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal probe slot calculation.
pub trait SeasonalProbeSlotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalProbeSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalProbeSlotAdjuster {
    pub step: f64,
}

impl SeasonalProbeSlotFactor for RoundingSeasonalProbeSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalProbeSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionSeasonalProbeSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalProbeSlotFactor for RegionSeasonalProbeSlotAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &SeasonalProbeSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalProbeSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalProbeSlotFactor for TieredSeasonalProbeSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalProbeSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalProbeSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalProbeSlotFactor for FlaggedSeasonalProbeSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalProbeSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalProbeSlotAdjuster {
    pub percent: f64,
}

impl SeasonalProbeSlotFactor for PercentSeasonalProbeSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalProbeSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalProbeSlotAdjuster {
    pub minimum: f64,
}

impl SeasonalProbeSlotFactor for MinimumSeasonalProbeSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalProbeSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the seasonal probe slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalProbeSlotEngine {
    pub steps: Vec<Rc<dyn SeasonalProbeSlotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalProbeSlotEngine {
    pub fn evaluate(&mut self, item: &SeasonalProbeSlot) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalProbeSlotFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalProbeSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_seasonal_probe_slot_engine(flags: &flags::Registry) -> SeasonalProbeSlotEngine {
    SeasonalProbeSlotEngine {
        steps: vec![
            Rc::new(RoundingSeasonalProbeSlotAdjuster { step: 1.0 }) as Rc<dyn SeasonalProbeSlotFactor>,
            Rc::new(RegionSeasonalProbeSlotAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.31), ("north".to_string(), 1.19)]),
            }),
            Rc::new(TieredSeasonalProbeSlotAdjuster {
                bounds: vec![381.0, 3882.0, 5413.0, 7417.0],
                rates: vec![1.14, 0.88, 0.98, 0.96],
            }),
            Rc::new(FlaggedSeasonalProbeSlotAdjuster {
                flags: flags.clone(),
                flag: "seasonal-probe-slot-rolling".to_string(),
                percent: -1.4,
            }),
            Rc::new(PercentSeasonalProbeSlotAdjuster { percent: 0.4 }),
            Rc::new(MinimumSeasonalProbeSlotAdjuster { minimum: 25.9 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 87037.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_probe_slot_engine(&flags::new_registry());
        let item = new_seasonal_probe_slot("code-93", "label-436");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
