use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default zone summary calculation.
pub trait DefaultZoneSummaryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultZoneSummary, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultZoneSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultZoneSummaryFactor for FlaggedDefaultZoneSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultZoneSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultZoneSummaryAdjuster {
    pub percent: f64,
}

impl DefaultZoneSummaryFactor for PercentDefaultZoneSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultZoneSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultZoneSummaryAdjuster {
    pub minimum: f64,
}

impl DefaultZoneSummaryFactor for MinimumDefaultZoneSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultZoneSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultZoneSummaryAdjuster {
    pub step: f64,
}

impl DefaultZoneSummaryFactor for RoundingDefaultZoneSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultZoneSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionDefaultZoneSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultZoneSummaryFactor for RegionDefaultZoneSummaryAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &DefaultZoneSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultZoneSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultZoneSummaryFactor for TieredDefaultZoneSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultZoneSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the default zone summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultZoneSummaryEngine {
    pub steps: Vec<Rc<dyn DefaultZoneSummaryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultZoneSummaryEngine {
    pub fn evaluate(&mut self, item: &DefaultZoneSummary) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultZoneSummaryFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultZoneSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_zone_summary_engine(flags: &flags::Registry) -> DefaultZoneSummaryEngine {
    DefaultZoneSummaryEngine {
        steps: vec![
            Rc::new(FlaggedDefaultZoneSummaryAdjuster {
                flags: flags.clone(),
                flag: "default-zone-summary-weighted".to_string(),
                percent: -4.7,
            }) as Rc<dyn DefaultZoneSummaryFactor>,
            Rc::new(PercentDefaultZoneSummaryAdjuster { percent: 3.0 }),
            Rc::new(MinimumDefaultZoneSummaryAdjuster { minimum: 44.0 }),
            Rc::new(RoundingDefaultZoneSummaryAdjuster { step: 0.05 }),
            Rc::new(RegionDefaultZoneSummaryAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.16),
                    ("retail".to_string(), 1.2),
                    ("east".to_string(), 0.73),
                ]),
            }),
            Rc::new(TieredDefaultZoneSummaryAdjuster {
                bounds: vec![1384.0, 1937.0, 3274.0, 9023.0],
                rates: vec![1.14, 0.84, 0.95, 1.08],
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 61987.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_zone_summary_engine(&flags::new_registry());
        let item = new_default_zone_summary("code-6", "label-489");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
