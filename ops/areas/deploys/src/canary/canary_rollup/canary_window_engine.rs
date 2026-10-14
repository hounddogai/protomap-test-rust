use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the canary window calculation.
pub trait CanaryWindowModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanaryWindow, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedCanaryWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CanaryWindowModifier for FlaggedCanaryWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CanaryWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCanaryWindowAdjuster {
    pub percent: f64,
}

impl CanaryWindowModifier for PercentCanaryWindowAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CanaryWindow, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelCanaryWindowAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CanaryWindowModifier for ChannelCanaryWindowAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &CanaryWindow, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanaryWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CanaryWindowModifier for TieredCanaryWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CanaryWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the canary window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanaryWindowEngine {
    pub steps: Vec<Rc<dyn CanaryWindowModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanaryWindowEngine {
    pub fn price(&mut self, item: &CanaryWindow) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[CanaryWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_canary_window_engine(flags: &flags::Registry) -> CanaryWindowEngine {
    CanaryWindowEngine {
        steps: vec![
            Rc::new(FlaggedCanaryWindowAdjuster {
                flags: flags.clone(),
                flag: "canary-window-lenient".to_string(),
                percent: 5.1,
            }) as Rc<dyn CanaryWindowModifier>,
            Rc::new(PercentCanaryWindowAdjuster { percent: 19.6 }),
            Rc::new(ChannelCanaryWindowAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.12),
                    ("north".to_string(), 0.97),
                    ("online".to_string(), 0.82),
                ]),
            }),
            Rc::new(TieredCanaryWindowAdjuster {
                bounds: vec![3818.0, 6807.0],
                rates: vec![0.94, 0.91],
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 22785.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_canary_window_engine(&flags::new_registry());
        let item = new_canary_window("code-867", "name-635");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
