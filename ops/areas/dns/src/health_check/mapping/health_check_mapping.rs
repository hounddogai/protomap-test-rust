use super::*;

use acme_kit::text;

/// Where a health check mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckMappingStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl HealthCheckMappingStatus {
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

impl std::fmt::Display for HealthCheckMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A health check mapping as the health check team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckMapping {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub updated_at: i64,
    pub tenant_id: String,
    pub status: HealthCheckMappingStatus,
}

impl HealthCheckMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckMappingStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckMappingStatus) -> bool {
        match self.status {
            HealthCheckMappingStatus::Idle => {
                return next == HealthCheckMappingStatus::Ready || next == HealthCheckMappingStatus::Stopped;
            }
            HealthCheckMappingStatus::Warming => {
                return next == HealthCheckMappingStatus::Ready;
            }
            HealthCheckMappingStatus::Ready => {
                return next == HealthCheckMappingStatus::Stopped;
            }
            HealthCheckMappingStatus::Draining => {
                return next == HealthCheckMappingStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: HealthCheckMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.updated_at as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckMapping {
        HealthCheckMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            updated_at: self.updated_at,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a health check mapping in status idle.
pub fn new_health_check_mapping(code: &str, title: &str) -> HealthCheckMapping {
    HealthCheckMapping {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        updated_at: 0,
        tenant_id: "".to_string(),
        status: HealthCheckMappingStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_health_check_mapping("code-836", "title-883");
        assert_eq!(item.status, HealthCheckMappingStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_health_check_mapping("code-836", "title-883");
        if item.move_to(HealthCheckMappingStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckMappingStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_health_check_mapping("code-836", "title-883");
        item.status = HealthCheckMappingStatus::Stopped;
        assert!(
            item.move_to(HealthCheckMappingStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
