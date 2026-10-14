use super::*;

use acme_kit::text;

/// Where a verifier baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierBaselineStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl VerifierBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for VerifierBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a verifier baseline and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBaseline {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub ttl_seconds: i64,
    pub score: f64,
    pub status: VerifierBaselineStatus,
}

impl VerifierBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierBaselineStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierBaselineStatus) -> bool {
        match self.status {
            VerifierBaselineStatus::Requested => {
                match next {
                    VerifierBaselineStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierBaselineStatus::Granted => {
                match next {
                    VerifierBaselineStatus::Revoked | VerifierBaselineStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierBaselineStatus::Revoked => {
                return next == VerifierBaselineStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: VerifierBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.score - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierBaseline {
        VerifierBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            ttl_seconds: self.ttl_seconds,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a verifier baseline in status requested.
pub fn new_verifier_baseline(code: &str, label: &str) -> VerifierBaseline {
    VerifierBaseline {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        ttl_seconds: 0,
        score: 0.0,
        status: VerifierBaselineStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_verifier_baseline("code-540", "label-220");
        assert_eq!(item.status, VerifierBaselineStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_verifier_baseline("code-540", "label-220");
        if item.move_to(VerifierBaselineStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierBaselineStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_verifier_baseline("code-540", "label-220");
        item.status = VerifierBaselineStatus::Expired;
        assert!(
            item.move_to(VerifierBaselineStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
