use super::*;

use acme_kit::text;

/// Where a fallback redaction preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackRedactionPreferenceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FallbackRedactionPreferenceStatus {
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

impl std::fmt::Display for FallbackRedactionPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback redaction preference as the redaction team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionPreference {
    pub code: String,
    pub label: String,
    pub quota_units: i64,
    pub description: String,
    pub quantity: i64,
    pub status: FallbackRedactionPreferenceStatus,
}

impl FallbackRedactionPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackRedactionPreferenceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackRedactionPreferenceStatus) -> bool {
        match self.status {
            FallbackRedactionPreferenceStatus::Idle => {
                return next == FallbackRedactionPreferenceStatus::Draining
                    || next == FallbackRedactionPreferenceStatus::Stopped;
            }
            FallbackRedactionPreferenceStatus::Warming => {
                return next == FallbackRedactionPreferenceStatus::Stopped;
            }
            FallbackRedactionPreferenceStatus::Ready => {
                match next {
                    FallbackRedactionPreferenceStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRedactionPreferenceStatus::Draining => {
                return next == FallbackRedactionPreferenceStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackRedactionPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback redaction preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.quantity as f64) * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackRedactionPreference {
        FallbackRedactionPreference {
            code: self.code.clone(),
            label: self.label.clone(),
            quota_units: self.quota_units,
            description: self.description.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a fallback redaction preference in status idle.
pub fn new_fallback_redaction_preference(code: &str, label: &str) -> FallbackRedactionPreference {
    FallbackRedactionPreference {
        code: code.to_string(),
        label: label.to_string(),
        quota_units: 0,
        description: "".to_string(),
        quantity: 0,
        status: FallbackRedactionPreferenceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_fallback_redaction_preference("code-860", "label-885");
        assert_eq!(item.status, FallbackRedactionPreferenceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_fallback_redaction_preference("code-860", "label-885");
        if item.move_to(FallbackRedactionPreferenceStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackRedactionPreferenceStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_fallback_redaction_preference("code-860", "label-885");
        item.status = FallbackRedactionPreferenceStatus::Stopped;
        assert!(
            item.move_to(FallbackRedactionPreferenceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
