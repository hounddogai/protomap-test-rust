use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the zone summary calculation.
pub trait ZoneSummaryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ZoneSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSummaryExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl ZoneSummaryAdjuster for ZoneSummaryExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &ZoneSummary, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceZoneSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ZoneSummaryAdjuster for SourceZoneSummaryAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &ZoneSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingZoneSummaryAdjuster {
    pub step: f64,
}

impl ZoneSummaryAdjuster for RoundingZoneSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ZoneSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentZoneSummaryAdjuster {
    pub percent: f64,
}

impl ZoneSummaryAdjuster for PercentZoneSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ZoneSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumZoneSummaryAdjuster {
    pub minimum: f64,
}

impl ZoneSummaryAdjuster for MinimumZoneSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ZoneSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the zone summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ZoneSummaryEngine {
    pub steps: Vec<Rc<dyn ZoneSummaryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ZoneSummaryEngine {
    pub fn run(&mut self, item: &ZoneSummary) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ZoneSummaryAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ZoneSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_zone_summary_engine() -> ZoneSummaryEngine {
    ZoneSummaryEngine {
        steps: vec![
            Rc::new(ZoneSummaryExpiresAtOffsetAdjuster { weight: 0.11 }) as Rc<dyn ZoneSummaryAdjuster>,
            Rc::new(SourceZoneSummaryAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.31), ("south".to_string(), 0.89)]),
            }),
            Rc::new(RoundingZoneSummaryAdjuster { step: 0.05 }),
            Rc::new(PercentZoneSummaryAdjuster { percent: 7.5 }),
            Rc::new(MinimumZoneSummaryAdjuster { minimum: 2.9 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 71024.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_zone_summary_engine();
        let item = new_zone_summary("code-75", "label-880");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
