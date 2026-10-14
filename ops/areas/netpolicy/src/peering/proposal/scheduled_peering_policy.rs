use super::*;

use acme_kit::text;

/// Where a scheduled peering policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledPeeringPolicyStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ScheduledPeeringPolicyStatus {
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

impl std::fmt::Display for ScheduledPeeringPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled peering policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPeeringPolicy {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub pinned: bool,
    pub tags: Vec<String>,
    pub rate: f64,
    pub window_days: i64,
    pub ceiling: f64,
    pub status: ScheduledPeeringPolicyStatus,
}

impl ScheduledPeeringPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledPeeringPolicyStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledPeeringPolicyStatus) -> bool {
        match self.status {
            ScheduledPeeringPolicyStatus::Planned => {
                match next {
                    ScheduledPeeringPolicyStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledPeeringPolicyStatus::Scheduled => {
                match next {
                    ScheduledPeeringPolicyStatus::InProgress | ScheduledPeeringPolicyStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledPeeringPolicyStatus::InProgress => {
                match next {
                    ScheduledPeeringPolicyStatus::Completed | ScheduledPeeringPolicyStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledPeeringPolicyStatus::Completed => {
                match next {
                    ScheduledPeeringPolicyStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ScheduledPeeringPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled peering policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.duration_minutes as f64) * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledPeeringPolicy {
        ScheduledPeeringPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            pinned: self.pinned,
            tags: self.tags.clone(),
            rate: self.rate,
            window_days: self.window_days,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled peering policy in status planned.
pub fn new_scheduled_peering_policy(code: &str, label: &str) -> ScheduledPeeringPolicy {
    ScheduledPeeringPolicy {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        pinned: false,
        tags: Vec::new(),
        rate: 0.0,
        window_days: 0,
        ceiling: 0.0,
        status: ScheduledPeeringPolicyStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_scheduled_peering_policy("code-245", "label-200");
        assert_eq!(item.status, ScheduledPeeringPolicyStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_scheduled_peering_policy("code-245", "label-200");
        if item.move_to(ScheduledPeeringPolicyStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledPeeringPolicyStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_scheduled_peering_policy("code-245", "label-200");
        item.status = ScheduledPeeringPolicyStatus::Cancelled;
        assert!(
            item.move_to(ScheduledPeeringPolicyStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
