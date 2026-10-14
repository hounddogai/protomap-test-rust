use super::*;

use acme_kit::text;

/// Where a rolling segment entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingSegmentEntryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RollingSegmentEntryStatus {
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

impl std::fmt::Display for RollingSegmentEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling segment entry as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSegmentEntry {
    pub code: String,
    pub name: String,
    pub region: String,
    pub label: String,
    pub amount: f64,
    pub notes: String,
    pub external_ref: String,
    pub status: RollingSegmentEntryStatus,
}

impl RollingSegmentEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingSegmentEntryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingSegmentEntryStatus) -> bool {
        match self.status {
            RollingSegmentEntryStatus::Idle => {
                return next == RollingSegmentEntryStatus::Ready;
            }
            RollingSegmentEntryStatus::Warming => {
                return next == RollingSegmentEntryStatus::Ready;
            }
            RollingSegmentEntryStatus::Ready => {
                return next == RollingSegmentEntryStatus::Draining;
            }
            RollingSegmentEntryStatus::Draining => {
                match next {
                    RollingSegmentEntryStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RollingSegmentEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling segment entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> RollingSegmentEntry {
        RollingSegmentEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            label: self.label.clone(),
            amount: self.amount,
            notes: self.notes.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a rolling segment entry in status idle.
pub fn new_rolling_segment_entry(code: &str, name: &str) -> RollingSegmentEntry {
    RollingSegmentEntry {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        label: "".to_string(),
        amount: 0.0,
        notes: "".to_string(),
        external_ref: "".to_string(),
        status: RollingSegmentEntryStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_rolling_segment_entry("code-647", "name-424");
        assert_eq!(item.status, RollingSegmentEntryStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_rolling_segment_entry("code-647", "name-424");
        if item.move_to(RollingSegmentEntryStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingSegmentEntryStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_rolling_segment_entry("code-647", "name-424");
        item.status = RollingSegmentEntryStatus::Stopped;
        assert!(
            item.move_to(RollingSegmentEntryStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
