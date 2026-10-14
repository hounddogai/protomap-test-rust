use super::*;

use acme_kit::text;

/// Where a legacy verifier manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyVerifierManifestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LegacyVerifierManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for LegacyVerifierManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy verifier manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyVerifierManifest {
    pub code: String,
    pub title: String,
    pub created_at: i64,
    pub attempts: i64,
    pub sequence: i64,
    pub host: String,
    pub ends_at: i64,
    pub external_ref: String,
    pub segment: String,
    pub grace_days: i64,
    pub status: LegacyVerifierManifestStatus,
}

impl LegacyVerifierManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyVerifierManifestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyVerifierManifestStatus) -> bool {
        match self.status {
            LegacyVerifierManifestStatus::New => {
                match next {
                    LegacyVerifierManifestStatus::Running | LegacyVerifierManifestStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyVerifierManifestStatus::Queued => {
                return next == LegacyVerifierManifestStatus::Running;
            }
            LegacyVerifierManifestStatus::Running => {
                return next == LegacyVerifierManifestStatus::Failed;
            }
            LegacyVerifierManifestStatus::Done => {
                return next == LegacyVerifierManifestStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyVerifierManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy verifier manifest {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.created_at as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyVerifierManifest {
        LegacyVerifierManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            attempts: self.attempts,
            sequence: self.sequence,
            host: self.host.clone(),
            ends_at: self.ends_at,
            external_ref: self.external_ref.clone(),
            segment: self.segment.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a legacy verifier manifest in status new.
pub fn new_legacy_verifier_manifest(code: &str, title: &str) -> LegacyVerifierManifest {
    LegacyVerifierManifest {
        code: code.to_string(),
        title: title.to_string(),
        created_at: 0,
        attempts: 0,
        sequence: 0,
        host: "".to_string(),
        ends_at: 0,
        external_ref: "".to_string(),
        segment: "".to_string(),
        grace_days: 0,
        status: LegacyVerifierManifestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_legacy_verifier_manifest("code-884", "title-474");
        assert_eq!(item.status, LegacyVerifierManifestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_legacy_verifier_manifest("code-884", "title-474");
        if item.move_to(LegacyVerifierManifestStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyVerifierManifestStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_legacy_verifier_manifest("code-884", "title-474");
        item.status = LegacyVerifierManifestStatus::Failed;
        assert!(
            item.move_to(LegacyVerifierManifestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
