use super::*;

use acme_kit::text;

/// Where a verifier override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierOverrideStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl VerifierOverrideStatus {
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

impl std::fmt::Display for VerifierOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a verifier override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierOverride {
    pub code: String,
    pub title: String,
    pub duration_minutes: i64,
    pub cluster: String,
    pub port: i64,
    pub retries: i64,
    pub limit: i64,
    pub namespace: String,
    pub weight: f64,
    pub status: VerifierOverrideStatus,
}

impl VerifierOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierOverrideStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierOverrideStatus) -> bool {
        match self.status {
            VerifierOverrideStatus::Idle => {
                return next == VerifierOverrideStatus::Draining;
            }
            VerifierOverrideStatus::Warming => {
                match next {
                    VerifierOverrideStatus::Ready | VerifierOverrideStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierOverrideStatus::Ready => {
                match next {
                    VerifierOverrideStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierOverrideStatus::Draining => {
                match next {
                    VerifierOverrideStatus::Stopped => {
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

    pub fn move_to(&mut self, next: VerifierOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "verifier override", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierOverride {
        VerifierOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            cluster: self.cluster.clone(),
            port: self.port,
            retries: self.retries,
            limit: self.limit,
            namespace: self.namespace.clone(),
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a verifier override in status idle.
pub fn new_verifier_override(code: &str, title: &str) -> VerifierOverride {
    VerifierOverride {
        code: code.to_string(),
        title: title.to_string(),
        duration_minutes: 0,
        cluster: "".to_string(),
        port: 0,
        retries: 0,
        limit: 0,
        namespace: "".to_string(),
        weight: 0.0,
        status: VerifierOverrideStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_verifier_override("code-804", "title-309");
        assert_eq!(item.status, VerifierOverrideStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_verifier_override("code-804", "title-309");
        if item.move_to(VerifierOverrideStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierOverrideStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_verifier_override("code-804", "title-309");
        item.status = VerifierOverrideStatus::Stopped;
        assert!(
            item.move_to(VerifierOverrideStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
