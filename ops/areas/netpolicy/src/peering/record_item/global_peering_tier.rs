use super::*;

use acme_kit::text;

/// Where a global peering tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalPeeringTierStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GlobalPeeringTierStatus {
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

impl std::fmt::Display for GlobalPeeringTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global peering tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringTier {
    pub code: String,
    pub title: String,
    pub visible: bool,
    pub quota_units: i64,
    pub port: i64,
    pub ttl_seconds: i64,
    pub contact_email: String,
    pub duration_minutes: i64,
    pub ratio: f64,
    pub status: GlobalPeeringTierStatus,
}

impl GlobalPeeringTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalPeeringTierStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalPeeringTierStatus) -> bool {
        match self.status {
            GlobalPeeringTierStatus::Planned => {
                return next == GlobalPeeringTierStatus::InProgress || next == GlobalPeeringTierStatus::Completed;
            }
            GlobalPeeringTierStatus::Scheduled => {
                match next {
                    GlobalPeeringTierStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalPeeringTierStatus::InProgress => {
                match next {
                    GlobalPeeringTierStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalPeeringTierStatus::Completed => {
                return next == GlobalPeeringTierStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalPeeringTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global peering tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "global peering tier", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.ttl_seconds as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalPeeringTier {
        GlobalPeeringTier {
            code: self.code.clone(),
            title: self.title.clone(),
            visible: self.visible,
            quota_units: self.quota_units,
            port: self.port,
            ttl_seconds: self.ttl_seconds,
            contact_email: self.contact_email.clone(),
            duration_minutes: self.duration_minutes,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a global peering tier in status planned.
pub fn new_global_peering_tier(code: &str, title: &str) -> GlobalPeeringTier {
    GlobalPeeringTier {
        code: code.to_string(),
        title: title.to_string(),
        visible: false,
        quota_units: 0,
        port: 0,
        ttl_seconds: 0,
        contact_email: "".to_string(),
        duration_minutes: 0,
        ratio: 0.0,
        status: GlobalPeeringTierStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_global_peering_tier("code-464", "title-724");
        assert_eq!(item.status, GlobalPeeringTierStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_global_peering_tier("code-464", "title-724");
        if item.move_to(GlobalPeeringTierStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalPeeringTierStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_global_peering_tier("code-464", "title-724");
        item.status = GlobalPeeringTierStatus::Cancelled;
        assert!(
            item.move_to(GlobalPeeringTierStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
