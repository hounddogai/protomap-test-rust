use super::*;

use acme_kit::text;

/// Where a pending canary bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingCanaryBundleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PendingCanaryBundleStatus {
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

impl std::fmt::Display for PendingCanaryBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending canary bundle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingCanaryBundle {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub limit: i64,
    pub country: String,
    pub updated_at: i64,
    pub key_id: String,
    pub threshold: f64,
    pub sequence: i64,
    pub status: PendingCanaryBundleStatus,
}

impl PendingCanaryBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingCanaryBundleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingCanaryBundleStatus) -> bool {
        match self.status {
            PendingCanaryBundleStatus::Idle => {
                return next == PendingCanaryBundleStatus::Stopped;
            }
            PendingCanaryBundleStatus::Warming => {
                return next == PendingCanaryBundleStatus::Ready;
            }
            PendingCanaryBundleStatus::Ready => {
                return next == PendingCanaryBundleStatus::Stopped;
            }
            PendingCanaryBundleStatus::Draining => {
                match next {
                    PendingCanaryBundleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PendingCanaryBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending canary bundle {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.threshold + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingCanaryBundle {
        PendingCanaryBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            limit: self.limit,
            country: self.country.clone(),
            updated_at: self.updated_at,
            key_id: self.key_id.clone(),
            threshold: self.threshold,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a pending canary bundle in status idle.
pub fn new_pending_canary_bundle(code: &str, label: &str) -> PendingCanaryBundle {
    PendingCanaryBundle {
        code: code.to_string(),
        label: label.to_string(),
        locale: "".to_string(),
        limit: 0,
        country: "".to_string(),
        updated_at: 0,
        key_id: "".to_string(),
        threshold: 0.0,
        sequence: 0,
        status: PendingCanaryBundleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_pending_canary_bundle("code-846", "label-744");
        assert_eq!(item.status, PendingCanaryBundleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_pending_canary_bundle("code-846", "label-744");
        if item.move_to(PendingCanaryBundleStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingCanaryBundleStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_pending_canary_bundle("code-846", "label-744");
        item.status = PendingCanaryBundleStatus::Stopped;
        assert!(
            item.move_to(PendingCanaryBundleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
