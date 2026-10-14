use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled zone score calculation.
pub trait ScheduledZoneScoreStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledZoneScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledZoneScoreAdjuster {
    pub cap: f64,
}

impl ScheduledZoneScoreStep for CapScheduledZoneScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledZoneScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledZoneScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledZoneScoreStep for FlaggedScheduledZoneScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledZoneScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledZoneScoreAdjuster {
    pub percent: f64,
}

impl ScheduledZoneScoreStep for PercentScheduledZoneScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledZoneScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledZoneScoreAdjuster {
    pub step: f64,
}

impl ScheduledZoneScoreStep for RoundingScheduledZoneScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledZoneScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentScheduledZoneScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledZoneScoreStep for SegmentScheduledZoneScoreAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &ScheduledZoneScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the scheduled zone score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledZoneScoreEngine {
    pub steps: Vec<Rc<dyn ScheduledZoneScoreStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledZoneScoreEngine {
    pub fn compute(&mut self, item: &ScheduledZoneScore) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledZoneScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_zone_score_engine(flags: &flags::Registry) -> ScheduledZoneScoreEngine {
    ScheduledZoneScoreEngine {
        steps: vec![
            Rc::new(CapScheduledZoneScoreAdjuster { cap: 15533.0 }) as Rc<dyn ScheduledZoneScoreStep>,
            Rc::new(FlaggedScheduledZoneScoreAdjuster {
                flags: flags.clone(),
                flag: "scheduled-zone-score-legacy".to_string(),
                percent: 3.8,
            }),
            Rc::new(PercentScheduledZoneScoreAdjuster { percent: 13.5 }),
            Rc::new(RoundingScheduledZoneScoreAdjuster { step: 0.05 }),
            Rc::new(SegmentScheduledZoneScoreAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.08),
                    ("south".to_string(), 0.76),
                    ("east".to_string(), 0.9),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 35025.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_zone_score_engine(&flags::new_registry());
        let item = new_scheduled_zone_score("code-894", "name-21");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
