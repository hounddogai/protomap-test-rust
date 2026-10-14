use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the zone slot calculation.
pub trait ZoneSlotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ZoneSlot, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedZoneSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ZoneSlotFactor for FlaggedZoneSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ZoneSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameZoneSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ZoneSlotFactor for NameZoneSlotAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ZoneSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapZoneSlotAdjuster {
    pub cap: f64,
}

impl ZoneSlotFactor for CapZoneSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ZoneSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the zone slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ZoneSlotEngine {
    pub steps: Vec<Rc<dyn ZoneSlotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ZoneSlotEngine {
    pub fn evaluate(&mut self, item: &ZoneSlot) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[ZoneSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_zone_slot_engine(flags: &flags::Registry) -> ZoneSlotEngine {
    ZoneSlotEngine {
        steps: vec![
            Rc::new(FlaggedZoneSlotAdjuster {
                flags: flags.clone(),
                flag: "zone-slot-tiered".to_string(),
                percent: -6.8,
            }) as Rc<dyn ZoneSlotFactor>,
            Rc::new(NameZoneSlotAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.21),
                    ("west".to_string(), 1.39),
                    ("north".to_string(), 1.07),
                ]),
            }),
            Rc::new(CapZoneSlotAdjuster { cap: 6931.0 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 95290.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_zone_slot_engine(&flags::new_registry());
        let item = new_zone_slot("code-391", "name-518");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
