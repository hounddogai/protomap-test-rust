use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the scheduled region pin preference calculation.
pub trait ScheduledRegionPinPreferenceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledRegionPinPreference, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledRegionPinPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledRegionPinPreferenceAdjuster for FlaggedScheduledRegionPinPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledRegionPinPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledRegionPinPreferenceAdjuster {
    pub percent: f64,
}

impl ScheduledRegionPinPreferenceAdjuster for PercentScheduledRegionPinPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledRegionPinPreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledRegionPinPreferenceAdjuster {
    pub minimum: f64,
}

impl ScheduledRegionPinPreferenceAdjuster for MinimumScheduledRegionPinPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledRegionPinPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRegionPinPreferenceLimitOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledRegionPinPreferenceAdjuster for ScheduledRegionPinPreferenceLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledRegionPinPreference, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentScheduledRegionPinPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledRegionPinPreferenceAdjuster for SegmentScheduledRegionPinPreferenceAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &ScheduledRegionPinPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledRegionPinPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledRegionPinPreferenceAdjuster for TieredScheduledRegionPinPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledRegionPinPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the scheduled region pin preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledRegionPinPreferenceEngine {
    pub steps: Vec<Rc<dyn ScheduledRegionPinPreferenceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledRegionPinPreferenceEngine {
    pub fn price(&mut self, item: &ScheduledRegionPinPreference) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledRegionPinPreferenceAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledRegionPinPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_region_pin_preference_engine(flags: &flags::Registry) -> ScheduledRegionPinPreferenceEngine {
    ScheduledRegionPinPreferenceEngine {
        steps: vec![
            Rc::new(FlaggedScheduledRegionPinPreferenceAdjuster {
                flags: flags.clone(),
                flag: "scheduled-region-pin-preference-capped".to_string(),
                percent: 6.7,
            }) as Rc<dyn ScheduledRegionPinPreferenceAdjuster>,
            Rc::new(PercentScheduledRegionPinPreferenceAdjuster { percent: -21.8 }),
            Rc::new(MinimumScheduledRegionPinPreferenceAdjuster { minimum: 24.4 }),
            Rc::new(ScheduledRegionPinPreferenceLimitOffsetAdjuster { weight: 1.45 }),
            Rc::new(SegmentScheduledRegionPinPreferenceAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.31),
                    ("north".to_string(), 1.36),
                    ("south".to_string(), 0.99),
                ]),
            }),
            Rc::new(TieredScheduledRegionPinPreferenceAdjuster {
                bounds: vec![2219.0, 5881.0],
                rates: vec![0.83, 1.08],
            }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 91458.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_region_pin_preference_engine(&flags::new_registry());
        let item = new_scheduled_region_pin_preference("code-620", "title-60");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
