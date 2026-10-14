use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a fallback verifier audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackVerifierAuditStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FallbackVerifierAuditStatus {
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

impl std::fmt::Display for FallbackVerifierAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback verifier audit as the verifier team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackVerifierAudit {
    pub code: String,
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub tags: Vec<String>,
    pub rotation_days: i64,
    pub attempts: i64,
    pub port: i64,
    pub minimum: f64,
    pub fingerprint: String,
    pub weight: f64,
    pub segment: String,
    pub status: FallbackVerifierAuditStatus,
}

impl FallbackVerifierAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackVerifierAuditStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackVerifierAuditStatus) -> bool {
        match self.status {
            FallbackVerifierAuditStatus::Idle => {
                return next == FallbackVerifierAuditStatus::Ready;
            }
            FallbackVerifierAuditStatus::Warming => {
                match next {
                    FallbackVerifierAuditStatus::Ready | FallbackVerifierAuditStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackVerifierAuditStatus::Ready => {
                return next == FallbackVerifierAuditStatus::Draining || next == FallbackVerifierAuditStatus::Stopped;
            }
            FallbackVerifierAuditStatus::Draining => {
                return next == FallbackVerifierAuditStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackVerifierAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback verifier audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback verifier audit", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.attempts as f64) * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackVerifierAudit {
        FallbackVerifierAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            tags: self.tags.clone(),
            rotation_days: self.rotation_days,
            attempts: self.attempts,
            port: self.port,
            minimum: self.minimum,
            fingerprint: self.fingerprint.clone(),
            weight: self.weight,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a fallback verifier audit in status idle.
pub fn new_fallback_verifier_audit(code: &str, name: &str) -> FallbackVerifierAudit {
    FallbackVerifierAudit {
        code: code.to_string(),
        name: name.to_string(),
        attributes: BTreeMap::new(),
        tags: Vec::new(),
        rotation_days: 0,
        attempts: 0,
        port: 0,
        minimum: 0.0,
        fingerprint: "".to_string(),
        weight: 0.0,
        segment: "".to_string(),
        status: FallbackVerifierAuditStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_fallback_verifier_audit("code-961", "name-365");
        assert_eq!(item.status, FallbackVerifierAuditStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_fallback_verifier_audit("code-961", "name-365");
        if item.move_to(FallbackVerifierAuditStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackVerifierAuditStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_fallback_verifier_audit("code-961", "name-365");
        item.status = FallbackVerifierAuditStatus::Stopped;
        assert!(
            item.move_to(FallbackVerifierAuditStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
