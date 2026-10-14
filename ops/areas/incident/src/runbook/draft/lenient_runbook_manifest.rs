use super::*;

use acme_kit::text;

/// Where a lenient runbook manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientRunbookManifestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientRunbookManifestStatus {
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

impl std::fmt::Display for LenientRunbookManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient runbook manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRunbookManifest {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub pinned: bool,
    pub margin: f64,
    pub cipher: String,
    pub ends_at: i64,
    pub ttl_seconds: i64,
    pub checksum: String,
    pub contact_email: String,
    pub capacity: i64,
    pub status: LenientRunbookManifestStatus,
}

impl LenientRunbookManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientRunbookManifestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientRunbookManifestStatus) -> bool {
        match self.status {
            LenientRunbookManifestStatus::Idle => {
                match next {
                    LenientRunbookManifestStatus::Warming | LenientRunbookManifestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRunbookManifestStatus::Warming => {
                match next {
                    LenientRunbookManifestStatus::Draining | LenientRunbookManifestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRunbookManifestStatus::Ready => {
                match next {
                    LenientRunbookManifestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRunbookManifestStatus::Draining => {
                return next == LenientRunbookManifestStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientRunbookManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient runbook manifest {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.window_days as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientRunbookManifest {
        LenientRunbookManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            pinned: self.pinned,
            margin: self.margin,
            cipher: self.cipher.clone(),
            ends_at: self.ends_at,
            ttl_seconds: self.ttl_seconds,
            checksum: self.checksum.clone(),
            contact_email: self.contact_email.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a lenient runbook manifest in status idle.
pub fn new_lenient_runbook_manifest(code: &str, name: &str) -> LenientRunbookManifest {
    LenientRunbookManifest {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        pinned: false,
        margin: 0.0,
        cipher: "".to_string(),
        ends_at: 0,
        ttl_seconds: 0,
        checksum: "".to_string(),
        contact_email: "".to_string(),
        capacity: 0,
        status: LenientRunbookManifestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_lenient_runbook_manifest("code-60", "name-947");
        assert_eq!(item.status, LenientRunbookManifestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_lenient_runbook_manifest("code-60", "name-947");
        if item.move_to(LenientRunbookManifestStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientRunbookManifestStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_lenient_runbook_manifest("code-60", "name-947");
        item.status = LenientRunbookManifestStatus::Stopped;
        assert!(
            item.move_to(LenientRunbookManifestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
