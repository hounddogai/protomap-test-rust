use super::*;

use acme_kit::text;

/// Where a health check variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckVarianceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl HealthCheckVarianceStatus {
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

impl std::fmt::Display for HealthCheckVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A health check variance as the health check team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckVariance {
    pub code: String,
    pub title: String,
    pub created_at: i64,
    pub approved: bool,
    pub cost: f64,
    pub segment: String,
    pub floor_value: f64,
    pub audience: String,
    pub discount: f64,
    pub unit: String,
    pub grace_days: i64,
    pub status: HealthCheckVarianceStatus,
}

impl HealthCheckVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckVarianceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckVarianceStatus) -> bool {
        match self.status {
            HealthCheckVarianceStatus::Idle => {
                return next == HealthCheckVarianceStatus::Warming || next == HealthCheckVarianceStatus::Stopped;
            }
            HealthCheckVarianceStatus::Warming => {
                match next {
                    HealthCheckVarianceStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckVarianceStatus::Ready => {
                return next == HealthCheckVarianceStatus::Draining;
            }
            HealthCheckVarianceStatus::Draining => {
                return next == HealthCheckVarianceStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: HealthCheckVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        (self.floor_value * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckVariance {
        HealthCheckVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            approved: self.approved,
            cost: self.cost,
            segment: self.segment.clone(),
            floor_value: self.floor_value,
            audience: self.audience.clone(),
            discount: self.discount,
            unit: self.unit.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a health check variance in status idle.
pub fn new_health_check_variance(code: &str, title: &str) -> HealthCheckVariance {
    HealthCheckVariance {
        code: code.to_string(),
        title: title.to_string(),
        created_at: 0,
        approved: false,
        cost: 0.0,
        segment: "".to_string(),
        floor_value: 0.0,
        audience: "".to_string(),
        discount: 0.0,
        unit: "".to_string(),
        grace_days: 0,
        status: HealthCheckVarianceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_health_check_variance("code-177", "title-165");
        assert_eq!(item.status, HealthCheckVarianceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_health_check_variance("code-177", "title-165");
        if item.move_to(HealthCheckVarianceStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckVarianceStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_health_check_variance("code-177", "title-165");
        item.status = HealthCheckVarianceStatus::Stopped;
        assert!(
            item.move_to(HealthCheckVarianceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
