use super::*;

use acme_kit::text;

/// Where a slo quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloQuotaStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SloQuotaStatus {
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

impl std::fmt::Display for SloQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One slo quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SloQuota {
    pub code: String,
    pub title: String,
    pub revision: i64,
    pub quota_units: i64,
    pub minimum: f64,
    pub expires_at: i64,
    pub discount: f64,
    pub ratio: f64,
    pub attempts: i64,
    pub status: SloQuotaStatus,
}

impl SloQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloQuotaStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloQuotaStatus) -> bool {
        match self.status {
            SloQuotaStatus::Idle => {
                return next == SloQuotaStatus::Ready || next == SloQuotaStatus::Stopped;
            }
            SloQuotaStatus::Warming => {
                match next {
                    SloQuotaStatus::Ready | SloQuotaStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloQuotaStatus::Ready => {
                return next == SloQuotaStatus::Draining;
            }
            SloQuotaStatus::Draining => {
                return next == SloQuotaStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SloQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.attempts as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloQuota {
        SloQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            revision: self.revision,
            quota_units: self.quota_units,
            minimum: self.minimum,
            expires_at: self.expires_at,
            discount: self.discount,
            ratio: self.ratio,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a slo quota in status idle.
pub fn new_slo_quota(code: &str, title: &str) -> SloQuota {
    SloQuota {
        code: code.to_string(),
        title: title.to_string(),
        revision: 0,
        quota_units: 0,
        minimum: 0.0,
        expires_at: 0,
        discount: 0.0,
        ratio: 0.0,
        attempts: 0,
        status: SloQuotaStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_slo_quota("code-216", "title-635");
        assert_eq!(item.status, SloQuotaStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_slo_quota("code-216", "title-635");
        if item.move_to(SloQuotaStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloQuotaStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_slo_quota("code-216", "title-635");
        item.status = SloQuotaStatus::Stopped;
        assert!(item.move_to(SloQuotaStatus::Idle).is_err(), "expected move to fail");
    }
}
