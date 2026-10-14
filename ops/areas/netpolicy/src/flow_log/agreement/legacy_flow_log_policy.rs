use super::*;

use acme_kit::text;

/// Where a legacy flow log policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyFlowLogPolicyStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LegacyFlowLogPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LegacyFlowLogPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy flow log policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogPolicy {
    pub code: String,
    pub label: String,
    pub window_days: i64,
    pub attempts: i64,
    pub expires_at: i64,
    pub burst: i64,
    pub namespace: String,
    pub cluster: String,
    pub visible: bool,
    pub timezone: String,
    pub status: LegacyFlowLogPolicyStatus,
}

impl LegacyFlowLogPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyFlowLogPolicyStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyFlowLogPolicyStatus) -> bool {
        match self.status {
            LegacyFlowLogPolicyStatus::Created => {
                match next {
                    LegacyFlowLogPolicyStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyFlowLogPolicyStatus::Validated => {
                match next {
                    LegacyFlowLogPolicyStatus::Published | LegacyFlowLogPolicyStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyFlowLogPolicyStatus::Published => {
                return next == LegacyFlowLogPolicyStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyFlowLogPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy flow log policy {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.expires_at as f64 + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyFlowLogPolicy {
        LegacyFlowLogPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            window_days: self.window_days,
            attempts: self.attempts,
            expires_at: self.expires_at,
            burst: self.burst,
            namespace: self.namespace.clone(),
            cluster: self.cluster.clone(),
            visible: self.visible,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a legacy flow log policy in status created.
pub fn new_legacy_flow_log_policy(code: &str, label: &str) -> LegacyFlowLogPolicy {
    LegacyFlowLogPolicy {
        code: code.to_string(),
        label: label.to_string(),
        window_days: 0,
        attempts: 0,
        expires_at: 0,
        burst: 0,
        namespace: "".to_string(),
        cluster: "".to_string(),
        visible: false,
        timezone: "".to_string(),
        status: LegacyFlowLogPolicyStatus::Created,
    }
}
