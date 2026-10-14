use super::*;

use acme_kit::text;

/// Where a pending usage quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingUsageQuotaStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PendingUsageQuotaStatus {
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

impl std::fmt::Display for PendingUsageQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending usage quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingUsageQuota {
    pub code: String,
    pub label: String,
    pub revision: i64,
    pub burst: i64,
    pub rotation_days: i64,
    pub quantity: i64,
    pub channel: String,
    pub sequence: i64,
    pub floor_value: f64,
    pub source: String,
    pub status: PendingUsageQuotaStatus,
}

impl PendingUsageQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingUsageQuotaStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingUsageQuotaStatus) -> bool {
        match self.status {
            PendingUsageQuotaStatus::Idle => {
                return next == PendingUsageQuotaStatus::Stopped;
            }
            PendingUsageQuotaStatus::Warming => {
                return next == PendingUsageQuotaStatus::Draining;
            }
            PendingUsageQuotaStatus::Ready => {
                match next {
                    PendingUsageQuotaStatus::Draining | PendingUsageQuotaStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingUsageQuotaStatus::Draining => {
                match next {
                    PendingUsageQuotaStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingUsageQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending usage quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.rotation_days as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingUsageQuota {
        PendingUsageQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            revision: self.revision,
            burst: self.burst,
            rotation_days: self.rotation_days,
            quantity: self.quantity,
            channel: self.channel.clone(),
            sequence: self.sequence,
            floor_value: self.floor_value,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a pending usage quota in status idle.
pub fn new_pending_usage_quota(code: &str, label: &str) -> PendingUsageQuota {
    PendingUsageQuota {
        code: code.to_string(),
        label: label.to_string(),
        revision: 0,
        burst: 0,
        rotation_days: 0,
        quantity: 0,
        channel: "".to_string(),
        sequence: 0,
        floor_value: 0.0,
        source: "".to_string(),
        status: PendingUsageQuotaStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_pending_usage_quota("code-882", "label-622");
        assert_eq!(item.status, PendingUsageQuotaStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_pending_usage_quota("code-882", "label-622");
        if item.move_to(PendingUsageQuotaStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingUsageQuotaStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_pending_usage_quota("code-882", "label-622");
        item.status = PendingUsageQuotaStatus::Stopped;
        assert!(
            item.move_to(PendingUsageQuotaStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
