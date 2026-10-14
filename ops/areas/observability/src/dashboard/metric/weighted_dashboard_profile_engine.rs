use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the weighted dashboard profile calculation.
pub trait WeightedDashboardProfileModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedDashboardProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedDashboardProfileAdjuster {
    pub cap: f64,
}

impl WeightedDashboardProfileModifier for CapWeightedDashboardProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedDashboardProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedDashboardProfileAdjuster {
    pub minimum: f64,
}

impl WeightedDashboardProfileModifier for MinimumWeightedDashboardProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedDashboardProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedDashboardProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedDashboardProfileModifier for TieredWeightedDashboardProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedDashboardProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedDashboardProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedDashboardProfileModifier for FlaggedWeightedDashboardProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedDashboardProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelWeightedDashboardProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedDashboardProfileModifier for ChannelWeightedDashboardProfileAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &WeightedDashboardProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedDashboardProfileAdjuster {
    pub percent: f64,
}

impl WeightedDashboardProfileModifier for PercentWeightedDashboardProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedDashboardProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted dashboard profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedDashboardProfileEngine {
    pub steps: Vec<Rc<dyn WeightedDashboardProfileModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedDashboardProfileEngine {
    pub fn price(&mut self, item: &WeightedDashboardProfile) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedDashboardProfileModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedDashboardProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_dashboard_profile_engine(flags: &flags::Registry) -> WeightedDashboardProfileEngine {
    WeightedDashboardProfileEngine {
        steps: vec![
            Rc::new(CapWeightedDashboardProfileAdjuster { cap: 1530.0 }) as Rc<dyn WeightedDashboardProfileModifier>,
            Rc::new(MinimumWeightedDashboardProfileAdjuster { minimum: 25.4 }),
            Rc::new(TieredWeightedDashboardProfileAdjuster {
                bounds: vec![360.0, 2271.0, 5892.0, 9228.0],
                rates: vec![1.09, 0.83, 1.15, 1.07],
            }),
            Rc::new(FlaggedWeightedDashboardProfileAdjuster {
                flags: flags.clone(),
                flag: "weighted-dashboard-profile-batched".to_string(),
                percent: -6.9,
            }),
            Rc::new(ChannelWeightedDashboardProfileAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.13),
                    ("central".to_string(), 0.85),
                    ("online".to_string(), 1.37),
                ]),
            }),
            Rc::new(PercentWeightedDashboardProfileAdjuster { percent: -24.0 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 63268.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_dashboard_profile_engine(&flags::new_registry());
        let item = new_weighted_dashboard_profile("code-662", "title-25");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
