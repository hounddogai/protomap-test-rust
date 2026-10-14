use super::*;

use acme_kit::text;

/// Where a health check score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckScoreStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl HealthCheckScoreStatus {
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

impl std::fmt::Display for HealthCheckScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A health check score as the health check team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckScore {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub sequence: i64,
    pub key_id: String,
    pub region: String,
    pub ends_at: i64,
    pub retries: i64,
    pub burst: i64,
    pub principal: String,
    pub active: bool,
    pub status: HealthCheckScoreStatus,
}

impl HealthCheckScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckScoreStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckScoreStatus) -> bool {
        match self.status {
            HealthCheckScoreStatus::Idle => {
                match next {
                    HealthCheckScoreStatus::Draining | HealthCheckScoreStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckScoreStatus::Warming => {
                return next == HealthCheckScoreStatus::Draining;
            }
            HealthCheckScoreStatus::Ready => {
                return next == HealthCheckScoreStatus::Stopped;
            }
            HealthCheckScoreStatus::Draining => {
                return next == HealthCheckScoreStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: HealthCheckScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check score {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.burst as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckScore {
        HealthCheckScore {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            sequence: self.sequence,
            key_id: self.key_id.clone(),
            region: self.region.clone(),
            ends_at: self.ends_at,
            retries: self.retries,
            burst: self.burst,
            principal: self.principal.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a health check score in status idle.
pub fn new_health_check_score(code: &str, name: &str) -> HealthCheckScore {
    HealthCheckScore {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        sequence: 0,
        key_id: "".to_string(),
        region: "".to_string(),
        ends_at: 0,
        retries: 0,
        burst: 0,
        principal: "".to_string(),
        active: false,
        status: HealthCheckScoreStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_health_check_score("code-372", "name-340");
        assert_eq!(item.status, HealthCheckScoreStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_health_check_score("code-372", "name-340");
        if item.move_to(HealthCheckScoreStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckScoreStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_health_check_score("code-372", "name-340");
        item.status = HealthCheckScoreStatus::Stopped;
        assert!(
            item.move_to(HealthCheckScoreStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
