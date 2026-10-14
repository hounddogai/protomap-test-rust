use super::*;

use acme_kit::text;

/// Where a failover override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverOverrideStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FailoverOverrideStatus {
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

impl std::fmt::Display for FailoverOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A failover override as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverOverride {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub updated_at: i64,
    pub ratio: f64,
    pub principal: String,
    pub burst: i64,
    pub tenant_id: String,
    pub limit: i64,
    pub status: FailoverOverrideStatus,
}

impl FailoverOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverOverrideStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverOverrideStatus) -> bool {
        match self.status {
            FailoverOverrideStatus::Idle => {
                return next == FailoverOverrideStatus::Ready;
            }
            FailoverOverrideStatus::Warming => {
                return next == FailoverOverrideStatus::Ready || next == FailoverOverrideStatus::Stopped;
            }
            FailoverOverrideStatus::Ready => {
                match next {
                    FailoverOverrideStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverOverrideStatus::Draining => {
                return next == FailoverOverrideStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.updated_at as f64) * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverOverride {
        FailoverOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            updated_at: self.updated_at,
            ratio: self.ratio,
            principal: self.principal.clone(),
            burst: self.burst,
            tenant_id: self.tenant_id.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a failover override in status idle.
pub fn new_failover_override(code: &str, label: &str) -> FailoverOverride {
    FailoverOverride {
        code: code.to_string(),
        label: label.to_string(),
        issuer: "".to_string(),
        updated_at: 0,
        ratio: 0.0,
        principal: "".to_string(),
        burst: 0,
        tenant_id: "".to_string(),
        limit: 0,
        status: FailoverOverrideStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_failover_override("code-964", "label-598");
        assert_eq!(item.status, FailoverOverrideStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_failover_override("code-964", "label-598");
        if item.move_to(FailoverOverrideStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverOverrideStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_failover_override("code-964", "label-598");
        item.status = FailoverOverrideStatus::Stopped;
        assert!(
            item.move_to(FailoverOverrideStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
