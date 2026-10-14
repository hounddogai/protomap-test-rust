use super::*;

use acme_kit::text;

/// Where a measurement slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementSlotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl MeasurementSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for MeasurementSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a measurement slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementSlot {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub tenant_id: String,
    pub checksum: String,
    pub cost: f64,
    pub description: String,
    pub batch_size: i64,
    pub revision: i64,
    pub rate: f64,
    pub status: MeasurementSlotStatus,
}

impl MeasurementSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementSlotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementSlotStatus) -> bool {
        match self.status {
            MeasurementSlotStatus::New => {
                match next {
                    MeasurementSlotStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementSlotStatus::Queued => {
                return next == MeasurementSlotStatus::Running || next == MeasurementSlotStatus::Done;
            }
            MeasurementSlotStatus::Running => {
                return next == MeasurementSlotStatus::Done;
            }
            MeasurementSlotStatus::Done => {
                return next == MeasurementSlotStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MeasurementSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "measurement slot", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.rate - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementSlot {
        MeasurementSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            tenant_id: self.tenant_id.clone(),
            checksum: self.checksum.clone(),
            cost: self.cost,
            description: self.description.clone(),
            batch_size: self.batch_size,
            revision: self.revision,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a measurement slot in status new.
pub fn new_measurement_slot(code: &str, name: &str) -> MeasurementSlot {
    MeasurementSlot {
        code: code.to_string(),
        name: name.to_string(),
        notes: "".to_string(),
        tenant_id: "".to_string(),
        checksum: "".to_string(),
        cost: 0.0,
        description: "".to_string(),
        batch_size: 0,
        revision: 0,
        rate: 0.0,
        status: MeasurementSlotStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_measurement_slot("code-446", "name-699");
        assert_eq!(item.status, MeasurementSlotStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_measurement_slot("code-446", "name-699");
        if item.move_to(MeasurementSlotStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementSlotStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_measurement_slot("code-446", "name-699");
        item.status = MeasurementSlotStatus::Failed;
        assert!(
            item.move_to(MeasurementSlotStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
