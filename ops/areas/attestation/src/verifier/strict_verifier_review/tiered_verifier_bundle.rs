use super::*;

use acme_kit::text;

/// Where a tiered verifier bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredVerifierBundleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TieredVerifierBundleStatus {
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

impl std::fmt::Display for TieredVerifierBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered verifier bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierBundle {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub batch_size: i64,
    pub namespace: String,
    pub notes: String,
    pub window_days: i64,
    pub port: i64,
    pub ttl_seconds: i64,
    pub status: TieredVerifierBundleStatus,
}

impl TieredVerifierBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredVerifierBundleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredVerifierBundleStatus) -> bool {
        match self.status {
            TieredVerifierBundleStatus::Idle => {
                return next == TieredVerifierBundleStatus::Ready || next == TieredVerifierBundleStatus::Draining;
            }
            TieredVerifierBundleStatus::Warming => {
                return next == TieredVerifierBundleStatus::Ready || next == TieredVerifierBundleStatus::Stopped;
            }
            TieredVerifierBundleStatus::Ready => {
                return next == TieredVerifierBundleStatus::Draining || next == TieredVerifierBundleStatus::Stopped;
            }
            TieredVerifierBundleStatus::Draining => {
                match next {
                    TieredVerifierBundleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: TieredVerifierBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered verifier bundle {} cannot move from {} to {}",
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
        format!("{} {}", "tiered verifier bundle", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredVerifierBundle {
        TieredVerifierBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            batch_size: self.batch_size,
            namespace: self.namespace.clone(),
            notes: self.notes.clone(),
            window_days: self.window_days,
            port: self.port,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a tiered verifier bundle in status idle.
pub fn new_tiered_verifier_bundle(code: &str, label: &str) -> TieredVerifierBundle {
    TieredVerifierBundle {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        batch_size: 0,
        namespace: "".to_string(),
        notes: "".to_string(),
        window_days: 0,
        port: 0,
        ttl_seconds: 0,
        status: TieredVerifierBundleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_tiered_verifier_bundle("code-707", "label-638");
        assert_eq!(item.status, TieredVerifierBundleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_tiered_verifier_bundle("code-707", "label-638");
        if item.move_to(TieredVerifierBundleStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredVerifierBundleStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_tiered_verifier_bundle("code-707", "label-638");
        item.status = TieredVerifierBundleStatus::Stopped;
        assert!(
            item.move_to(TieredVerifierBundleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
