use super::*;

use acme_kit::text;

/// Where a fallback silence bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackSilenceBundleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FallbackSilenceBundleStatus {
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

impl std::fmt::Display for FallbackSilenceBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback silence bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSilenceBundle {
    pub code: String,
    pub title: String,
    pub discount: f64,
    pub namespace: String,
    pub country: String,
    pub version: i64,
    pub status: FallbackSilenceBundleStatus,
}

impl FallbackSilenceBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackSilenceBundleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackSilenceBundleStatus) -> bool {
        match self.status {
            FallbackSilenceBundleStatus::Idle => {
                return next == FallbackSilenceBundleStatus::Ready;
            }
            FallbackSilenceBundleStatus::Warming => {
                return next == FallbackSilenceBundleStatus::Stopped;
            }
            FallbackSilenceBundleStatus::Ready => {
                return next == FallbackSilenceBundleStatus::Stopped;
            }
            FallbackSilenceBundleStatus::Draining => {
                match next {
                    FallbackSilenceBundleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: FallbackSilenceBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback silence bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.discount + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackSilenceBundle {
        FallbackSilenceBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            discount: self.discount,
            namespace: self.namespace.clone(),
            country: self.country.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a fallback silence bundle in status idle.
pub fn new_fallback_silence_bundle(code: &str, title: &str) -> FallbackSilenceBundle {
    FallbackSilenceBundle {
        code: code.to_string(),
        title: title.to_string(),
        discount: 0.0,
        namespace: "".to_string(),
        country: "".to_string(),
        version: 0,
        status: FallbackSilenceBundleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_fallback_silence_bundle("code-272", "title-872");
        assert_eq!(item.status, FallbackSilenceBundleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_fallback_silence_bundle("code-272", "title-872");
        if item.move_to(FallbackSilenceBundleStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackSilenceBundleStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_fallback_silence_bundle("code-272", "title-872");
        item.status = FallbackSilenceBundleStatus::Stopped;
        assert!(
            item.move_to(FallbackSilenceBundleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
