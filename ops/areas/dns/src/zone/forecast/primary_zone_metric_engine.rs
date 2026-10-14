use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary zone metric calculation.
pub trait PrimaryZoneMetricFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryZoneMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitPrimaryZoneMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryZoneMetricFactor for UnitPrimaryZoneMetricAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &PrimaryZoneMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryZoneMetricAdjuster {
    pub cap: f64,
}

impl PrimaryZoneMetricFactor for CapPrimaryZoneMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryZoneMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryZoneMetricAdjuster {
    pub step: f64,
}

impl PrimaryZoneMetricFactor for RoundingPrimaryZoneMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryZoneMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryZoneMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryZoneMetricFactor for FlaggedPrimaryZoneMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryZoneMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary zone metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryZoneMetricEngine {
    pub steps: Vec<Rc<dyn PrimaryZoneMetricFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryZoneMetricEngine {
    pub fn price(&mut self, item: &PrimaryZoneMetric) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryZoneMetricFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryZoneMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_zone_metric_engine(flags: &flags::Registry) -> PrimaryZoneMetricEngine {
    PrimaryZoneMetricEngine {
        steps: vec![
            Rc::new(UnitPrimaryZoneMetricAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.37)]),
            }) as Rc<dyn PrimaryZoneMetricFactor>,
            Rc::new(CapPrimaryZoneMetricAdjuster { cap: 33174.0 }),
            Rc::new(RoundingPrimaryZoneMetricAdjuster { step: 0.5 }),
            Rc::new(FlaggedPrimaryZoneMetricAdjuster {
                flags: flags.clone(),
                flag: "primary-zone-metric-primary".to_string(),
                percent: 1.1,
            }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 53443.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_zone_metric_engine(&flags::new_registry());
        let item = new_primary_zone_metric("code-722", "name-104");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
