use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional suspension profile calculation.
pub trait RegionalSuspensionProfileModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalSuspensionProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterRegionalSuspensionProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalSuspensionProfileModifier for ClusterRegionalSuspensionProfileAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &RegionalSuspensionProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalSuspensionProfileAdjuster {
    pub percent: f64,
}

impl RegionalSuspensionProfileModifier for PercentRegionalSuspensionProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalSuspensionProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalSuspensionProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalSuspensionProfileModifier for FlaggedRegionalSuspensionProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalSuspensionProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSuspensionProfileLimitOffsetAdjuster {
    pub weight: f64,
}

impl RegionalSuspensionProfileModifier for RegionalSuspensionProfileLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalSuspensionProfile, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

/// Runs the regional suspension profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalSuspensionProfileEngine {
    pub steps: Vec<Rc<dyn RegionalSuspensionProfileModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalSuspensionProfileEngine {
    pub fn price(&mut self, item: &RegionalSuspensionProfile) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RegionalSuspensionProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_suspension_profile_engine(flags: &flags::Registry) -> RegionalSuspensionProfileEngine {
    RegionalSuspensionProfileEngine {
        steps: vec![
            Rc::new(ClusterRegionalSuspensionProfileAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.15)]),
            }) as Rc<dyn RegionalSuspensionProfileModifier>,
            Rc::new(PercentRegionalSuspensionProfileAdjuster { percent: 22.6 }),
            Rc::new(FlaggedRegionalSuspensionProfileAdjuster {
                flags: flags.clone(),
                flag: "regional-suspension-profile-legacy".to_string(),
                percent: -8.5,
            }),
            Rc::new(RegionalSuspensionProfileLimitOffsetAdjuster { weight: 1.91 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 84252.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_suspension_profile_engine(&flags::new_registry());
        let item = new_regional_suspension_profile("code-887", "title-425");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
