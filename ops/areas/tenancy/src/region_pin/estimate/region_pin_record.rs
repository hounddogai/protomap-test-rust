use super::*;

use acme_kit::text;

/// Where a region pin record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinRecordStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionPinRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RegionPinRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a region pin record and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRecord {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub rotation_days: i64,
    pub reviewer: String,
    pub status: RegionPinRecordStatus,
}

impl RegionPinRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinRecordStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinRecordStatus) -> bool {
        match self.status {
            RegionPinRecordStatus::Draft => {
                match next {
                    RegionPinRecordStatus::Pending | RegionPinRecordStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinRecordStatus::Pending => {
                return next == RegionPinRecordStatus::Active || next == RegionPinRecordStatus::Suspended;
            }
            RegionPinRecordStatus::Active => {
                return next == RegionPinRecordStatus::Suspended;
            }
            RegionPinRecordStatus::Suspended => {
                match next {
                    RegionPinRecordStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionPinRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.burst as f64 - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinRecord {
        RegionPinRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            rotation_days: self.rotation_days,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a region pin record in status draft.
pub fn new_region_pin_record(code: &str, label: &str) -> RegionPinRecord {
    RegionPinRecord {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        rotation_days: 0,
        reviewer: "".to_string(),
        status: RegionPinRecordStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_region_pin_record("code-940", "label-195");
        assert_eq!(item.status, RegionPinRecordStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_region_pin_record("code-940", "label-195");
        if item.move_to(RegionPinRecordStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinRecordStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_region_pin_record("code-940", "label-195");
        item.status = RegionPinRecordStatus::Closed;
        assert!(
            item.move_to(RegionPinRecordStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
