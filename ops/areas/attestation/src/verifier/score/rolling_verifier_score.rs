use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a rolling verifier score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingVerifierScoreStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RollingVerifierScoreStatus {
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

impl std::fmt::Display for RollingVerifierScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling verifier score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierScore {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub attributes: BTreeMap<String, String>,
    pub cluster: String,
    pub namespace: String,
    pub window_days: i64,
    pub contact_email: String,
    pub port: i64,
    pub grace_days: i64,
    pub score: f64,
    pub status: RollingVerifierScoreStatus,
}

impl RollingVerifierScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingVerifierScoreStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingVerifierScoreStatus) -> bool {
        match self.status {
            RollingVerifierScoreStatus::Idle => {
                match next {
                    RollingVerifierScoreStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingVerifierScoreStatus::Warming => {
                return next == RollingVerifierScoreStatus::Draining;
            }
            RollingVerifierScoreStatus::Ready => {
                return next == RollingVerifierScoreStatus::Stopped;
            }
            RollingVerifierScoreStatus::Draining => {
                match next {
                    RollingVerifierScoreStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RollingVerifierScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling verifier score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
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

    pub fn net_load(&self) -> f64 {
        ((self.score + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingVerifierScore {
        RollingVerifierScore {
            code: self.code.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            attributes: self.attributes.clone(),
            cluster: self.cluster.clone(),
            namespace: self.namespace.clone(),
            window_days: self.window_days,
            contact_email: self.contact_email.clone(),
            port: self.port,
            grace_days: self.grace_days,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a rolling verifier score in status idle.
pub fn new_rolling_verifier_score(code: &str, name: &str) -> RollingVerifierScore {
    RollingVerifierScore {
        code: code.to_string(),
        name: name.to_string(),
        external_ref: "".to_string(),
        attributes: BTreeMap::new(),
        cluster: "".to_string(),
        namespace: "".to_string(),
        window_days: 0,
        contact_email: "".to_string(),
        port: 0,
        grace_days: 0,
        score: 0.0,
        status: RollingVerifierScoreStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_rolling_verifier_score("code-808", "name-506");
        assert_eq!(item.status, RollingVerifierScoreStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_rolling_verifier_score("code-808", "name-506");
        if item.move_to(RollingVerifierScoreStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingVerifierScoreStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_rolling_verifier_score("code-808", "name-506");
        item.status = RollingVerifierScoreStatus::Stopped;
        assert!(
            item.move_to(RollingVerifierScoreStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
