use super::*;

use acme_kit::text;

/// Where a adaptive verifier baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveVerifierBaselineStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AdaptiveVerifierBaselineStatus {
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

impl std::fmt::Display for AdaptiveVerifierBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive verifier baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveVerifierBaseline {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub country: String,
    pub external_ref: String,
    pub expires_at: i64,
    pub tags: Vec<String>,
    pub status: AdaptiveVerifierBaselineStatus,
}

impl AdaptiveVerifierBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveVerifierBaselineStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveVerifierBaselineStatus) -> bool {
        match self.status {
            AdaptiveVerifierBaselineStatus::Idle => {
                return next == AdaptiveVerifierBaselineStatus::Warming
                    || next == AdaptiveVerifierBaselineStatus::Stopped;
            }
            AdaptiveVerifierBaselineStatus::Warming => {
                return next == AdaptiveVerifierBaselineStatus::Draining;
            }
            AdaptiveVerifierBaselineStatus::Ready => {
                match next {
                    AdaptiveVerifierBaselineStatus::Draining | AdaptiveVerifierBaselineStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveVerifierBaselineStatus::Draining => {
                match next {
                    AdaptiveVerifierBaselineStatus::Stopped => {
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

    pub fn move_to(&mut self, next: AdaptiveVerifierBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive verifier baseline {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_volume(&self) -> f64 {
        ((self.expires_at as f64) * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveVerifierBaseline {
        AdaptiveVerifierBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            country: self.country.clone(),
            external_ref: self.external_ref.clone(),
            expires_at: self.expires_at,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a adaptive verifier baseline in status idle.
pub fn new_adaptive_verifier_baseline(code: &str, title: &str) -> AdaptiveVerifierBaseline {
    AdaptiveVerifierBaseline {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        country: "".to_string(),
        external_ref: "".to_string(),
        expires_at: 0,
        tags: Vec::new(),
        status: AdaptiveVerifierBaselineStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_adaptive_verifier_baseline("code-864", "title-688");
        assert_eq!(item.status, AdaptiveVerifierBaselineStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_adaptive_verifier_baseline("code-864", "title-688");
        if item.move_to(AdaptiveVerifierBaselineStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveVerifierBaselineStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_adaptive_verifier_baseline("code-864", "title-688");
        item.status = AdaptiveVerifierBaselineStatus::Stopped;
        assert!(
            item.move_to(AdaptiveVerifierBaselineStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
