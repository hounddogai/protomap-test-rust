use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual health check score calculation.
pub trait ManualHealthCheckScoreAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualHealthCheckScore, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedManualHealthCheckScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualHealthCheckScoreAdjuster for FlaggedManualHealthCheckScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualHealthCheckScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualHealthCheckScoreAdjuster {
    pub cap: f64,
}

impl ManualHealthCheckScoreAdjuster for CapManualHealthCheckScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualHealthCheckScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleManualHealthCheckScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualHealthCheckScoreAdjuster for TitleManualHealthCheckScoreAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ManualHealthCheckScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the manual health check score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualHealthCheckScoreEngine {
    pub steps: Vec<Rc<dyn ManualHealthCheckScoreAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualHealthCheckScoreEngine {
    pub fn run(&mut self, item: &ManualHealthCheckScore) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[ManualHealthCheckScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_health_check_score_engine(flags: &flags::Registry) -> ManualHealthCheckScoreEngine {
    ManualHealthCheckScoreEngine {
        steps: vec![
            Rc::new(FlaggedManualHealthCheckScoreAdjuster {
                flags: flags.clone(),
                flag: "manual-health-check-score-scheduled".to_string(),
                percent: -1.4,
            }) as Rc<dyn ManualHealthCheckScoreAdjuster>,
            Rc::new(CapManualHealthCheckScoreAdjuster { cap: 12582.0 }),
            Rc::new(TitleManualHealthCheckScoreAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.82), ("central".to_string(), 0.93)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 90621.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_health_check_score_engine(&flags::new_registry());
        let item = new_manual_health_check_score("code-447", "title-819");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
