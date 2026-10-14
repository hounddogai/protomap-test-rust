use super::*;

use acme_kit::text;

/// Where a seasonal probe slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalProbeSlotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SeasonalProbeSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for SeasonalProbeSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal probe slot as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeSlot {
    pub code: String,
    pub label: String,
    pub principal: String,
    pub notes: String,
    pub region: String,
    pub reviewer: String,
    pub sequence: i64,
    pub rotation_days: i64,
    pub budget: f64,
    pub status: SeasonalProbeSlotStatus,
}

impl SeasonalProbeSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalProbeSlotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalProbeSlotStatus) -> bool {
        match self.status {
            SeasonalProbeSlotStatus::Idle => {
                match next {
                    SeasonalProbeSlotStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalProbeSlotStatus::Warming => {
                match next {
                    SeasonalProbeSlotStatus::Ready | SeasonalProbeSlotStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalProbeSlotStatus::Ready => {
                match next {
                    SeasonalProbeSlotStatus::Draining | SeasonalProbeSlotStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalProbeSlotStatus::Draining => {
                return next == SeasonalProbeSlotStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalProbeSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal probe slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.budget - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalProbeSlot {
        SeasonalProbeSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            principal: self.principal.clone(),
            notes: self.notes.clone(),
            region: self.region.clone(),
            reviewer: self.reviewer.clone(),
            sequence: self.sequence,
            rotation_days: self.rotation_days,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a seasonal probe slot in status idle.
pub fn new_seasonal_probe_slot(code: &str, label: &str) -> SeasonalProbeSlot {
    SeasonalProbeSlot {
        code: code.to_string(),
        label: label.to_string(),
        principal: "".to_string(),
        notes: "".to_string(),
        region: "".to_string(),
        reviewer: "".to_string(),
        sequence: 0,
        rotation_days: 0,
        budget: 0.0,
        status: SeasonalProbeSlotStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_seasonal_probe_slot("code-93", "label-436");
        assert_eq!(item.status, SeasonalProbeSlotStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_seasonal_probe_slot("code-93", "label-436");
        if item.move_to(SeasonalProbeSlotStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalProbeSlotStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_seasonal_probe_slot("code-93", "label-436");
        item.status = SeasonalProbeSlotStatus::Stopped;
        assert!(
            item.move_to(SeasonalProbeSlotStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
