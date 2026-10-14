use super::*;

use acme_kit::text;

/// Where a region pin profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinProfileStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RegionPinProfileStatus {
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

impl std::fmt::Display for RegionPinProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a region pin profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinProfile {
    pub code: String,
    pub name: String,
    pub pinned: bool,
    pub ends_at: i64,
    pub channel: String,
    pub status: RegionPinProfileStatus,
}

impl RegionPinProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinProfileStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinProfileStatus) -> bool {
        match self.status {
            RegionPinProfileStatus::Planned => {
                return next == RegionPinProfileStatus::InProgress;
            }
            RegionPinProfileStatus::Scheduled => {
                return next == RegionPinProfileStatus::Completed || next == RegionPinProfileStatus::Cancelled;
            }
            RegionPinProfileStatus::InProgress => {
                return next == RegionPinProfileStatus::Completed || next == RegionPinProfileStatus::Cancelled;
            }
            RegionPinProfileStatus::Completed => {
                return next == RegionPinProfileStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionPinProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "region pin profile", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> RegionPinProfile {
        RegionPinProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            pinned: self.pinned,
            ends_at: self.ends_at,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a region pin profile in status planned.
pub fn new_region_pin_profile(code: &str, name: &str) -> RegionPinProfile {
    RegionPinProfile {
        code: code.to_string(),
        name: name.to_string(),
        pinned: false,
        ends_at: 0,
        channel: "".to_string(),
        status: RegionPinProfileStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_region_pin_profile("code-363", "name-855");
        assert_eq!(item.status, RegionPinProfileStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_region_pin_profile("code-363", "name-855");
        if item.move_to(RegionPinProfileStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinProfileStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_region_pin_profile("code-363", "name-855");
        item.status = RegionPinProfileStatus::Cancelled;
        assert!(
            item.move_to(RegionPinProfileStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
