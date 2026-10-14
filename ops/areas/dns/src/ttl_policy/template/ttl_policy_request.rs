use super::*;

use acme_kit::text;

/// Where a ttl policy request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyRequestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TtlPolicyRequestStatus {
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

impl std::fmt::Display for TtlPolicyRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A ttl policy request. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyRequest {
    pub code: String,
    pub title: String,
    pub port: i64,
    pub description: String,
    pub tenant_id: String,
    pub threshold: f64,
    pub status: TtlPolicyRequestStatus,
}

impl TtlPolicyRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyRequestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyRequestStatus) -> bool {
        match self.status {
            TtlPolicyRequestStatus::Idle => {
                return next == TtlPolicyRequestStatus::Warming || next == TtlPolicyRequestStatus::Stopped;
            }
            TtlPolicyRequestStatus::Warming => {
                match next {
                    TtlPolicyRequestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyRequestStatus::Ready => {
                match next {
                    TtlPolicyRequestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyRequestStatus::Draining => {
                return next == TtlPolicyRequestStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.threshold - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyRequest {
        TtlPolicyRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            port: self.port,
            description: self.description.clone(),
            tenant_id: self.tenant_id.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a ttl policy request in status idle.
pub fn new_ttl_policy_request(code: &str, title: &str) -> TtlPolicyRequest {
    TtlPolicyRequest {
        code: code.to_string(),
        title: title.to_string(),
        port: 0,
        description: "".to_string(),
        tenant_id: "".to_string(),
        threshold: 0.0,
        status: TtlPolicyRequestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_ttl_policy_request("code-471", "title-984");
        assert_eq!(item.status, TtlPolicyRequestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_ttl_policy_request("code-471", "title-984");
        if item.move_to(TtlPolicyRequestStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyRequestStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_ttl_policy_request("code-471", "title-984");
        item.status = TtlPolicyRequestStatus::Stopped;
        assert!(
            item.move_to(TtlPolicyRequestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
