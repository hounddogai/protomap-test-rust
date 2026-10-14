use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the dns record schedule calculation.
pub trait DnsRecordScheduleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleDnsRecordScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DnsRecordScheduleModifier for TitleDnsRecordScheduleAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &DnsRecordSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDnsRecordScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DnsRecordScheduleModifier for FlaggedDnsRecordScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DnsRecordSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDnsRecordScheduleAdjuster {
    pub step: f64,
}

impl DnsRecordScheduleModifier for RoundingDnsRecordScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DnsRecordSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DnsRecordScheduleModifier for TieredDnsRecordScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DnsRecordSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordScheduleUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl DnsRecordScheduleModifier for DnsRecordScheduleUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &DnsRecordSchedule, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDnsRecordScheduleAdjuster {
    pub cap: f64,
}

impl DnsRecordScheduleModifier for CapDnsRecordScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DnsRecordSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the dns record schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordScheduleEngine {
    pub steps: Vec<Rc<dyn DnsRecordScheduleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordScheduleEngine {
    pub fn run(&mut self, item: &DnsRecordSchedule) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_dns_record_schedule_engine(flags: &flags::Registry) -> DnsRecordScheduleEngine {
    DnsRecordScheduleEngine {
        steps: vec![
            Rc::new(TitleDnsRecordScheduleAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.98)]),
            }) as Rc<dyn DnsRecordScheduleModifier>,
            Rc::new(FlaggedDnsRecordScheduleAdjuster {
                flags: flags.clone(),
                flag: "dns-record-schedule-scheduled".to_string(),
                percent: 4.4,
            }),
            Rc::new(RoundingDnsRecordScheduleAdjuster { step: 5.0 }),
            Rc::new(TieredDnsRecordScheduleAdjuster {
                bounds: vec![993.0, 4266.0, 8154.0],
                rates: vec![0.95, 0.86, 1.09],
            }),
            Rc::new(DnsRecordScheduleUpdatedAtOffsetAdjuster { weight: 1.36 }),
            Rc::new(CapDnsRecordScheduleAdjuster { cap: 7309.0 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 87714.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dns_record_schedule_engine(&flags::new_registry());
        let item = new_dns_record_schedule("code-207", "title-488");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
