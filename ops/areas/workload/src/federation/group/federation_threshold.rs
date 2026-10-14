use super::*;

use acme_kit::text;

/// Where a federation threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationThresholdStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FederationThresholdStatus {
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

impl std::fmt::Display for FederationThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One federation threshold, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationThreshold {
    pub code: String,
    pub label: String,
    pub cost: f64,
    pub archived: bool,
    pub starts_at: i64,
    pub active: bool,
    pub grace_days: i64,
    pub timezone: String,
    pub status: FederationThresholdStatus,
}

impl FederationThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationThresholdStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationThresholdStatus) -> bool {
        match self.status {
            FederationThresholdStatus::Planned => {
                return next == FederationThresholdStatus::Scheduled || next == FederationThresholdStatus::Cancelled;
            }
            FederationThresholdStatus::Scheduled => {
                match next {
                    FederationThresholdStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationThresholdStatus::InProgress => {
                return next == FederationThresholdStatus::Completed || next == FederationThresholdStatus::Cancelled;
            }
            FederationThresholdStatus::Completed => {
                return next == FederationThresholdStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.cost + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationThreshold {
        FederationThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            cost: self.cost,
            archived: self.archived,
            starts_at: self.starts_at,
            active: self.active,
            grace_days: self.grace_days,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a federation threshold in status planned.
pub fn new_federation_threshold(code: &str, label: &str) -> FederationThreshold {
    FederationThreshold {
        code: code.to_string(),
        label: label.to_string(),
        cost: 0.0,
        archived: false,
        starts_at: 0,
        active: false,
        grace_days: 0,
        timezone: "".to_string(),
        status: FederationThresholdStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_federation_threshold("code-801", "label-332");
        assert_eq!(item.status, FederationThresholdStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_federation_threshold("code-801", "label-332");
        if item.move_to(FederationThresholdStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationThresholdStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_federation_threshold("code-801", "label-332");
        item.status = FederationThresholdStatus::Cancelled;
        assert!(
            item.move_to(FederationThresholdStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
