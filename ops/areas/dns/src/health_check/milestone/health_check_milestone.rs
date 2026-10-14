use super::*;

use acme_kit::text;

/// Where a health check milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckMilestoneStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl HealthCheckMilestoneStatus {
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

impl std::fmt::Display for HealthCheckMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A health check milestone as the health check team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckMilestone {
    pub code: String,
    pub label: String,
    pub key_id: String,
    pub quantity: i64,
    pub priority: i64,
    pub starts_at: i64,
    pub timezone: String,
    pub fingerprint: String,
    pub segment: String,
    pub region: String,
    pub currency: String,
    pub status: HealthCheckMilestoneStatus,
}

impl HealthCheckMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckMilestoneStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckMilestoneStatus) -> bool {
        match self.status {
            HealthCheckMilestoneStatus::Idle => {
                match next {
                    HealthCheckMilestoneStatus::Warming | HealthCheckMilestoneStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckMilestoneStatus::Warming => {
                return next == HealthCheckMilestoneStatus::Ready || next == HealthCheckMilestoneStatus::Draining;
            }
            HealthCheckMilestoneStatus::Ready => {
                match next {
                    HealthCheckMilestoneStatus::Draining | HealthCheckMilestoneStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckMilestoneStatus::Draining => {
                return next == HealthCheckMilestoneStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: HealthCheckMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check milestone {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.priority as f64 - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckMilestone {
        HealthCheckMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            key_id: self.key_id.clone(),
            quantity: self.quantity,
            priority: self.priority,
            starts_at: self.starts_at,
            timezone: self.timezone.clone(),
            fingerprint: self.fingerprint.clone(),
            segment: self.segment.clone(),
            region: self.region.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a health check milestone in status idle.
pub fn new_health_check_milestone(code: &str, label: &str) -> HealthCheckMilestone {
    HealthCheckMilestone {
        code: code.to_string(),
        label: label.to_string(),
        key_id: "".to_string(),
        quantity: 0,
        priority: 0,
        starts_at: 0,
        timezone: "".to_string(),
        fingerprint: "".to_string(),
        segment: "".to_string(),
        region: "".to_string(),
        currency: "".to_string(),
        status: HealthCheckMilestoneStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_health_check_milestone("code-871", "label-402");
        assert_eq!(item.status, HealthCheckMilestoneStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_health_check_milestone("code-871", "label-402");
        if item.move_to(HealthCheckMilestoneStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckMilestoneStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_health_check_milestone("code-871", "label-402");
        item.status = HealthCheckMilestoneStatus::Stopped;
        assert!(
            item.move_to(HealthCheckMilestoneStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
