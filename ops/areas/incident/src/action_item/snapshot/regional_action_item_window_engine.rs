use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional action item window calculation.
pub trait RegionalActionItemWindowFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalActionItemWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalActionItemWindowAdjuster {
    pub step: f64,
}

impl RegionalActionItemWindowFactor for RoundingRegionalActionItemWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalActionItemWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemWindowPortOffsetAdjuster {
    pub weight: f64,
}

impl RegionalActionItemWindowFactor for RegionalActionItemWindowPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalActionItemWindow, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalActionItemWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalActionItemWindowFactor for FlaggedRegionalActionItemWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalActionItemWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelRegionalActionItemWindowAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalActionItemWindowFactor for ChannelRegionalActionItemWindowAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &RegionalActionItemWindow, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalActionItemWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalActionItemWindowFactor for TieredRegionalActionItemWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalActionItemWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the regional action item window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalActionItemWindowEngine {
    pub steps: Vec<Rc<dyn RegionalActionItemWindowFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalActionItemWindowEngine {
    pub fn price(&mut self, item: &RegionalActionItemWindow) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalActionItemWindowFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalActionItemWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_action_item_window_engine(flags: &flags::Registry) -> RegionalActionItemWindowEngine {
    RegionalActionItemWindowEngine {
        steps: vec![
            Rc::new(RoundingRegionalActionItemWindowAdjuster { step: 0.25 }) as Rc<dyn RegionalActionItemWindowFactor>,
            Rc::new(RegionalActionItemWindowPortOffsetAdjuster { weight: 0.62 }),
            Rc::new(FlaggedRegionalActionItemWindowAdjuster {
                flags: flags.clone(),
                flag: "regional-action-item-window-capped".to_string(),
                percent: -7.6,
            }),
            Rc::new(ChannelRegionalActionItemWindowAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.95), ("east".to_string(), 1.02)]),
            }),
            Rc::new(TieredRegionalActionItemWindowAdjuster {
                bounds: vec![140.0, 2692.0, 2829.0, 7010.0],
                rates: vec![1.01, 1.09, 1.09, 1.18],
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 27755.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_action_item_window_engine(&flags::new_registry());
        let item = new_regional_action_item_window("code-250", "label-91");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
