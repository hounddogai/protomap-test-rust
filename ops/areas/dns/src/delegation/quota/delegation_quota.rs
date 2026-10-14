use super::*;

use acme_kit::text;

/// Where a delegation quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationQuotaStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DelegationQuotaStatus {
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

impl std::fmt::Display for DelegationQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One delegation quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationQuota {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub created_at: i64,
    pub external_ref: String,
    pub notes: String,
    pub ttl_seconds: i64,
    pub burst: i64,
    pub expires_at: i64,
    pub ends_at: i64,
    pub status: DelegationQuotaStatus,
}

impl DelegationQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationQuotaStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationQuotaStatus) -> bool {
        match self.status {
            DelegationQuotaStatus::Idle => {
                match next {
                    DelegationQuotaStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationQuotaStatus::Warming => {
                match next {
                    DelegationQuotaStatus::Ready | DelegationQuotaStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationQuotaStatus::Ready => {
                return next == DelegationQuotaStatus::Draining || next == DelegationQuotaStatus::Stopped;
            }
            DelegationQuotaStatus::Draining => {
                return next == DelegationQuotaStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DelegationQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation quota {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.score - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationQuota {
        DelegationQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            created_at: self.created_at,
            external_ref: self.external_ref.clone(),
            notes: self.notes.clone(),
            ttl_seconds: self.ttl_seconds,
            burst: self.burst,
            expires_at: self.expires_at,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a delegation quota in status idle.
pub fn new_delegation_quota(code: &str, title: &str) -> DelegationQuota {
    DelegationQuota {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        created_at: 0,
        external_ref: "".to_string(),
        notes: "".to_string(),
        ttl_seconds: 0,
        burst: 0,
        expires_at: 0,
        ends_at: 0,
        status: DelegationQuotaStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_delegation_quota("code-331", "title-659");
        assert_eq!(item.status, DelegationQuotaStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_delegation_quota("code-331", "title-659");
        if item.move_to(DelegationQuotaStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationQuotaStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_delegation_quota("code-331", "title-659");
        item.status = DelegationQuotaStatus::Stopped;
        assert!(
            item.move_to(DelegationQuotaStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
