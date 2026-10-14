use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the timeline digest calculation.
pub trait TimelineDigestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TimelineDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTimelineDigestAdjuster {
    pub cap: f64,
}

impl TimelineDigestModifier for CapTimelineDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TimelineDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTimelineDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TimelineDigestModifier for TieredTimelineDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TimelineDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryTimelineDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TimelineDigestModifier for CountryTimelineDigestAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &TimelineDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTimelineDigestAdjuster {
    pub step: f64,
}

impl TimelineDigestModifier for RoundingTimelineDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TimelineDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTimelineDigestAdjuster {
    pub percent: f64,
}

impl TimelineDigestModifier for PercentTimelineDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TimelineDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the timeline digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TimelineDigestEngine {
    pub steps: Vec<Rc<dyn TimelineDigestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TimelineDigestEngine {
    pub fn run(&mut self, item: &TimelineDigest) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TimelineDigestModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TimelineDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_timeline_digest_engine() -> TimelineDigestEngine {
    TimelineDigestEngine {
        steps: vec![
            Rc::new(CapTimelineDigestAdjuster { cap: 38536.0 }) as Rc<dyn TimelineDigestModifier>,
            Rc::new(TieredTimelineDigestAdjuster {
                bounds: vec![5273.0, 5961.0],
                rates: vec![1.13, 1.18],
            }),
            Rc::new(CountryTimelineDigestAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.74)]),
            }),
            Rc::new(RoundingTimelineDigestAdjuster { step: 0.25 }),
            Rc::new(PercentTimelineDigestAdjuster { percent: 9.8 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 83001.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_timeline_digest_engine();
        let item = new_timeline_digest("code-707", "title-524");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
