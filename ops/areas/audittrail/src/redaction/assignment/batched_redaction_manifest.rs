use super::*;

use acme_kit::text;

/// Where a batched redaction manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRedactionManifestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BatchedRedactionManifestStatus {
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

impl std::fmt::Display for BatchedRedactionManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched redaction manifest as the redaction team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionManifest {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub audience: String,
    pub priority: i64,
    pub fingerprint: String,
    pub rate: f64,
    pub score: f64,
    pub retries: i64,
    pub status: BatchedRedactionManifestStatus,
}

impl BatchedRedactionManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRedactionManifestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRedactionManifestStatus) -> bool {
        match self.status {
            BatchedRedactionManifestStatus::Idle => {
                return next == BatchedRedactionManifestStatus::Draining
                    || next == BatchedRedactionManifestStatus::Stopped;
            }
            BatchedRedactionManifestStatus::Warming => {
                match next {
                    BatchedRedactionManifestStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRedactionManifestStatus::Ready => {
                return next == BatchedRedactionManifestStatus::Draining
                    || next == BatchedRedactionManifestStatus::Stopped;
            }
            BatchedRedactionManifestStatus::Draining => {
                return next == BatchedRedactionManifestStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedRedactionManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched redaction manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched redaction manifest", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        (self.rate * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedRedactionManifest {
        BatchedRedactionManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            audience: self.audience.clone(),
            priority: self.priority,
            fingerprint: self.fingerprint.clone(),
            rate: self.rate,
            score: self.score,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched redaction manifest in status idle.
pub fn new_batched_redaction_manifest(code: &str, name: &str) -> BatchedRedactionManifest {
    BatchedRedactionManifest {
        code: code.to_string(),
        name: name.to_string(),
        checksum: "".to_string(),
        audience: "".to_string(),
        priority: 0,
        fingerprint: "".to_string(),
        rate: 0.0,
        score: 0.0,
        retries: 0,
        status: BatchedRedactionManifestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_batched_redaction_manifest("code-69", "name-165");
        assert_eq!(item.status, BatchedRedactionManifestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_batched_redaction_manifest("code-69", "name-165");
        if item.move_to(BatchedRedactionManifestStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedRedactionManifestStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_batched_redaction_manifest("code-69", "name-165");
        item.status = BatchedRedactionManifestStatus::Stopped;
        assert!(
            item.move_to(BatchedRedactionManifestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
