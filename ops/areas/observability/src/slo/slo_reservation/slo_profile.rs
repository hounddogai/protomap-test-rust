use super::*;

use acme_kit::text;

/// Where a slo profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloProfileStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SloProfileStatus {
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

impl std::fmt::Display for SloProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A slo profile as the slo team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SloProfile {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub minimum: f64,
    pub active: bool,
    pub margin: f64,
    pub reviewer: String,
    pub duration_minutes: i64,
    pub discount: f64,
    pub tenant_id: String,
    pub status: SloProfileStatus,
}

impl SloProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloProfileStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloProfileStatus) -> bool {
        match self.status {
            SloProfileStatus::Planned => {
                return next == SloProfileStatus::InProgress;
            }
            SloProfileStatus::Scheduled => {
                return next == SloProfileStatus::Completed || next == SloProfileStatus::Cancelled;
            }
            SloProfileStatus::InProgress => {
                return next == SloProfileStatus::Cancelled;
            }
            SloProfileStatus::Completed => {
                return next == SloProfileStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SloProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo profile {} cannot move from {} to {}",
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
        ((self.discount + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloProfile {
        SloProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            segment: self.segment.clone(),
            minimum: self.minimum,
            active: self.active,
            margin: self.margin,
            reviewer: self.reviewer.clone(),
            duration_minutes: self.duration_minutes,
            discount: self.discount,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a slo profile in status planned.
pub fn new_slo_profile(code: &str, label: &str) -> SloProfile {
    SloProfile {
        code: code.to_string(),
        label: label.to_string(),
        segment: "".to_string(),
        minimum: 0.0,
        active: false,
        margin: 0.0,
        reviewer: "".to_string(),
        duration_minutes: 0,
        discount: 0.0,
        tenant_id: "".to_string(),
        status: SloProfileStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_slo_profile("code-164", "label-379");
        assert_eq!(item.status, SloProfileStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_slo_profile("code-164", "label-379");
        if item.move_to(SloProfileStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloProfileStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_slo_profile("code-164", "label-379");
        item.status = SloProfileStatus::Cancelled;
        assert!(
            item.move_to(SloProfileStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
