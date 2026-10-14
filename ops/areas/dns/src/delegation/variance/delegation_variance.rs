use super::*;

use acme_kit::text;

/// Where a delegation variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationVarianceStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DelegationVarianceStatus {
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

impl std::fmt::Display for DelegationVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One delegation variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationVariance {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub ttl_seconds: i64,
    pub ceiling: f64,
    pub timezone: String,
    pub duration_minutes: i64,
    pub channel: String,
    pub amount: f64,
    pub audience: String,
    pub status: DelegationVarianceStatus,
}

impl DelegationVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationVarianceStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationVarianceStatus) -> bool {
        match self.status {
            DelegationVarianceStatus::Planned => {
                match next {
                    DelegationVarianceStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationVarianceStatus::Scheduled => {
                match next {
                    DelegationVarianceStatus::Completed | DelegationVarianceStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationVarianceStatus::InProgress => {
                match next {
                    DelegationVarianceStatus::Completed | DelegationVarianceStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationVarianceStatus::Completed => {
                match next {
                    DelegationVarianceStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: DelegationVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.ttl_seconds as f64 - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationVariance {
        DelegationVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            ttl_seconds: self.ttl_seconds,
            ceiling: self.ceiling,
            timezone: self.timezone.clone(),
            duration_minutes: self.duration_minutes,
            channel: self.channel.clone(),
            amount: self.amount,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a delegation variance in status planned.
pub fn new_delegation_variance(code: &str, label: &str) -> DelegationVariance {
    DelegationVariance {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        ttl_seconds: 0,
        ceiling: 0.0,
        timezone: "".to_string(),
        duration_minutes: 0,
        channel: "".to_string(),
        amount: 0.0,
        audience: "".to_string(),
        status: DelegationVarianceStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_delegation_variance("code-852", "label-637");
        assert_eq!(item.status, DelegationVarianceStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_delegation_variance("code-852", "label-637");
        if item.move_to(DelegationVarianceStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationVarianceStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_delegation_variance("code-852", "label-637");
        item.status = DelegationVarianceStatus::Cancelled;
        assert!(
            item.move_to(DelegationVarianceStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
