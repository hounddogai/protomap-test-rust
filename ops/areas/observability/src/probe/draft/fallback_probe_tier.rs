use super::*;

use acme_kit::text;

/// Where a fallback probe tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackProbeTierStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FallbackProbeTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FallbackProbeTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback probe tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackProbeTier {
    pub code: String,
    pub name: String,
    pub discount: f64,
    pub quota_units: i64,
    pub active: bool,
    pub expires_at: i64,
    pub label: String,
    pub attempts: i64,
    pub status: FallbackProbeTierStatus,
}

impl FallbackProbeTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackProbeTierStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackProbeTierStatus) -> bool {
        match self.status {
            FallbackProbeTierStatus::Proposed => {
                return next == FallbackProbeTierStatus::Approved || next == FallbackProbeTierStatus::Rejected;
            }
            FallbackProbeTierStatus::Approved => {
                match next {
                    FallbackProbeTierStatus::Rejected | FallbackProbeTierStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackProbeTierStatus::Rejected => {
                match next {
                    FallbackProbeTierStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FallbackProbeTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback probe tier {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "fallback probe tier", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.attempts as f64 - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackProbeTier {
        FallbackProbeTier {
            code: self.code.clone(),
            name: self.name.clone(),
            discount: self.discount,
            quota_units: self.quota_units,
            active: self.active,
            expires_at: self.expires_at,
            label: self.label.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a fallback probe tier in status proposed.
pub fn new_fallback_probe_tier(code: &str, name: &str) -> FallbackProbeTier {
    FallbackProbeTier {
        code: code.to_string(),
        name: name.to_string(),
        discount: 0.0,
        quota_units: 0,
        active: false,
        expires_at: 0,
        label: "".to_string(),
        attempts: 0,
        status: FallbackProbeTierStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_fallback_probe_tier("code-882", "name-699");
        assert_eq!(item.status, FallbackProbeTierStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_fallback_probe_tier("code-882", "name-699");
        if item.move_to(FallbackProbeTierStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackProbeTierStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_fallback_probe_tier("code-882", "name-699");
        item.status = FallbackProbeTierStatus::Withdrawn;
        assert!(
            item.move_to(FallbackProbeTierStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
