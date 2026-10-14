use super::*;

use acme_kit::text;

/// Where a health check claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckClaimStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl HealthCheckClaimStatus {
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

impl std::fmt::Display for HealthCheckClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One health check claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckClaim {
    pub code: String,
    pub label: String,
    pub description: String,
    pub tags: Vec<String>,
    pub notes: String,
    pub window_days: i64,
    pub status: HealthCheckClaimStatus,
}

impl HealthCheckClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckClaimStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckClaimStatus) -> bool {
        match self.status {
            HealthCheckClaimStatus::Idle => {
                return next == HealthCheckClaimStatus::Warming;
            }
            HealthCheckClaimStatus::Warming => {
                return next == HealthCheckClaimStatus::Ready || next == HealthCheckClaimStatus::Draining;
            }
            HealthCheckClaimStatus::Ready => {
                match next {
                    HealthCheckClaimStatus::Draining | HealthCheckClaimStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckClaimStatus::Draining => {
                return next == HealthCheckClaimStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: HealthCheckClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.window_days as f64
    }

    pub fn copy(&self) -> HealthCheckClaim {
        HealthCheckClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            tags: self.tags.clone(),
            notes: self.notes.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a health check claim in status idle.
pub fn new_health_check_claim(code: &str, label: &str) -> HealthCheckClaim {
    HealthCheckClaim {
        code: code.to_string(),
        label: label.to_string(),
        description: "".to_string(),
        tags: Vec::new(),
        notes: "".to_string(),
        window_days: 0,
        status: HealthCheckClaimStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_health_check_claim("code-90", "label-388");
        assert_eq!(item.status, HealthCheckClaimStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_health_check_claim("code-90", "label-388");
        if item.move_to(HealthCheckClaimStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckClaimStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_health_check_claim("code-90", "label-388");
        item.status = HealthCheckClaimStatus::Stopped;
        assert!(
            item.move_to(HealthCheckClaimStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
