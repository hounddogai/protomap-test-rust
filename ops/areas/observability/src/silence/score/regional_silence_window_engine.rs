use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the regional silence window calculation.
pub trait RegionalSilenceWindowAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalSilenceWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalSilenceWindowAdjuster {
    pub minimum: f64,
}

impl RegionalSilenceWindowAdjuster for MinimumRegionalSilenceWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalSilenceWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalSilenceWindowAdjuster {
    pub cap: f64,
}

impl RegionalSilenceWindowAdjuster for CapRegionalSilenceWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalSilenceWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalSilenceWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalSilenceWindowAdjuster for FlaggedRegionalSilenceWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalSilenceWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional silence window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalSilenceWindowEngine {
    pub steps: Vec<Rc<dyn RegionalSilenceWindowAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalSilenceWindowEngine {
    pub fn run(&mut self, item: &RegionalSilenceWindow) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalSilenceWindowAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalSilenceWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_silence_window_engine(flags: &flags::Registry) -> RegionalSilenceWindowEngine {
    RegionalSilenceWindowEngine {
        steps: vec![
            Rc::new(MinimumRegionalSilenceWindowAdjuster { minimum: 15.1 }) as Rc<dyn RegionalSilenceWindowAdjuster>,
            Rc::new(CapRegionalSilenceWindowAdjuster { cap: 47933.0 }),
            Rc::new(FlaggedRegionalSilenceWindowAdjuster {
                flags: flags.clone(),
                flag: "regional-silence-window-manual".to_string(),
                percent: -9.1,
            }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 61004.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_silence_window_engine(&flags::new_registry());
        let item = new_regional_silence_window("code-504", "name-675");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
