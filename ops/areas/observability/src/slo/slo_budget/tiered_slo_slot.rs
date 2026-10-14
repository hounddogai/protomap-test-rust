use super::*;

use acme_kit::text;

/// Where a tiered slo slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredSloSlotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TieredSloSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for TieredSloSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered slo slot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredSloSlot {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub port: i64,
    pub issuer: String,
    pub sequence: i64,
    pub checksum: String,
    pub capacity: i64,
    pub ratio: f64,
    pub threshold: f64,
    pub rate: f64,
    pub status: TieredSloSlotStatus,
}

impl TieredSloSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredSloSlotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredSloSlotStatus) -> bool {
        match self.status {
            TieredSloSlotStatus::Planned => {
                return next == TieredSloSlotStatus::InProgress;
            }
            TieredSloSlotStatus::Scheduled => {
                match next {
                    TieredSloSlotStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredSloSlotStatus::InProgress => {
                return next == TieredSloSlotStatus::Cancelled;
            }
            TieredSloSlotStatus::Completed => {
                match next {
                    TieredSloSlotStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: TieredSloSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered slo slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.threshold + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredSloSlot {
        TieredSloSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            port: self.port,
            issuer: self.issuer.clone(),
            sequence: self.sequence,
            checksum: self.checksum.clone(),
            capacity: self.capacity,
            ratio: self.ratio,
            threshold: self.threshold,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a tiered slo slot in status planned.
pub fn new_tiered_slo_slot(code: &str, label: &str) -> TieredSloSlot {
    TieredSloSlot {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        port: 0,
        issuer: "".to_string(),
        sequence: 0,
        checksum: "".to_string(),
        capacity: 0,
        ratio: 0.0,
        threshold: 0.0,
        rate: 0.0,
        status: TieredSloSlotStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_tiered_slo_slot("code-307", "label-838");
        assert_eq!(item.status, TieredSloSlotStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_tiered_slo_slot("code-307", "label-838");
        if item.move_to(TieredSloSlotStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredSloSlotStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_tiered_slo_slot("code-307", "label-838");
        item.status = TieredSloSlotStatus::Cancelled;
        assert!(
            item.move_to(TieredSloSlotStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
