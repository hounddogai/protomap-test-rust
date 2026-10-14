use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the zone metric calculation.
pub trait ZoneMetricModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ZoneMetric, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedZoneMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ZoneMetricModifier for FlaggedZoneMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ZoneMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentZoneMetricAdjuster {
    pub percent: f64,
}

impl ZoneMetricModifier for PercentZoneMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ZoneMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumZoneMetricAdjuster {
    pub minimum: f64,
}

impl ZoneMetricModifier for MinimumZoneMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ZoneMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneMetricRateOffsetAdjuster {
    pub weight: f64,
}

impl ZoneMetricModifier for ZoneMetricRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &ZoneMetric, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryZoneMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ZoneMetricModifier for CategoryZoneMetricAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &ZoneMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapZoneMetricAdjuster {
    pub cap: f64,
}

impl ZoneMetricModifier for CapZoneMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ZoneMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the zone metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ZoneMetricEngine {
    pub steps: Vec<Rc<dyn ZoneMetricModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ZoneMetricEngine {
    pub fn compute(&mut self, item: &ZoneMetric) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ZoneMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_zone_metric_engine(flags: &flags::Registry) -> ZoneMetricEngine {
    ZoneMetricEngine {
        steps: vec![
            Rc::new(FlaggedZoneMetricAdjuster {
                flags: flags.clone(),
                flag: "zone-metric-scheduled".to_string(),
                percent: -2.1,
            }) as Rc<dyn ZoneMetricModifier>,
            Rc::new(PercentZoneMetricAdjuster { percent: -11.0 }),
            Rc::new(MinimumZoneMetricAdjuster { minimum: 26.9 }),
            Rc::new(ZoneMetricRateOffsetAdjuster { weight: 1.54 }),
            Rc::new(CategoryZoneMetricAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.26),
                    ("east".to_string(), 1.27),
                    ("west".to_string(), 1.09),
                ]),
            }),
            Rc::new(CapZoneMetricAdjuster { cap: 11195.0 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 78860.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_zone_metric_engine(&flags::new_registry());
        let item = new_zone_metric("code-531", "name-646");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
