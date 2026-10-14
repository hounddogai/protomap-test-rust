use super::*;

use acme_kit::text;

/// Where a pending segment policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingSegmentPolicyStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PendingSegmentPolicyStatus {
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

impl std::fmt::Display for PendingSegmentPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending segment policy as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSegmentPolicy {
    pub code: String,
    pub label: String,
    pub source: String,
    pub pinned: bool,
    pub discount: f64,
    pub phone: String,
    pub ends_at: i64,
    pub batch_size: i64,
    pub status: PendingSegmentPolicyStatus,
}

impl PendingSegmentPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingSegmentPolicyStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingSegmentPolicyStatus) -> bool {
        match self.status {
            PendingSegmentPolicyStatus::Idle => {
                return next == PendingSegmentPolicyStatus::Ready;
            }
            PendingSegmentPolicyStatus::Warming => {
                return next == PendingSegmentPolicyStatus::Draining;
            }
            PendingSegmentPolicyStatus::Ready => {
                match next {
                    PendingSegmentPolicyStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingSegmentPolicyStatus::Draining => {
                match next {
                    PendingSegmentPolicyStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PendingSegmentPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending segment policy {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "pending segment policy", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.batch_size as f64 + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingSegmentPolicy {
        PendingSegmentPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            pinned: self.pinned,
            discount: self.discount,
            phone: self.phone.clone(),
            ends_at: self.ends_at,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a pending segment policy in status idle.
pub fn new_pending_segment_policy(code: &str, label: &str) -> PendingSegmentPolicy {
    PendingSegmentPolicy {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        pinned: false,
        discount: 0.0,
        phone: "".to_string(),
        ends_at: 0,
        batch_size: 0,
        status: PendingSegmentPolicyStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_pending_segment_policy("code-791", "label-986");
        assert_eq!(item.status, PendingSegmentPolicyStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_pending_segment_policy("code-791", "label-986");
        if item.move_to(PendingSegmentPolicyStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingSegmentPolicyStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_pending_segment_policy("code-791", "label-986");
        item.status = PendingSegmentPolicyStatus::Stopped;
        assert!(
            item.move_to(PendingSegmentPolicyStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
