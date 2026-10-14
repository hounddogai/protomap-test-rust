use super::*;

use acme_kit::text;

/// Where a adaptive allowlist draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveAllowlistDraftStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AdaptiveAllowlistDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AdaptiveAllowlistDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive allowlist draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAllowlistDraft {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub rotation_days: i64,
    pub currency: String,
    pub contact_email: String,
    pub cluster: String,
    pub attempts: i64,
    pub status: AdaptiveAllowlistDraftStatus,
}

impl AdaptiveAllowlistDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveAllowlistDraftStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveAllowlistDraftStatus) -> bool {
        match self.status {
            AdaptiveAllowlistDraftStatus::Proposed => {
                return next == AdaptiveAllowlistDraftStatus::Withdrawn;
            }
            AdaptiveAllowlistDraftStatus::Approved => {
                match next {
                    AdaptiveAllowlistDraftStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveAllowlistDraftStatus::Rejected => {
                match next {
                    AdaptiveAllowlistDraftStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: AdaptiveAllowlistDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive allowlist draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.attempts as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveAllowlistDraft {
        AdaptiveAllowlistDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            timezone: self.timezone.clone(),
            rotation_days: self.rotation_days,
            currency: self.currency.clone(),
            contact_email: self.contact_email.clone(),
            cluster: self.cluster.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a adaptive allowlist draft in status proposed.
pub fn new_adaptive_allowlist_draft(code: &str, label: &str) -> AdaptiveAllowlistDraft {
    AdaptiveAllowlistDraft {
        code: code.to_string(),
        label: label.to_string(),
        timezone: "".to_string(),
        rotation_days: 0,
        currency: "".to_string(),
        contact_email: "".to_string(),
        cluster: "".to_string(),
        attempts: 0,
        status: AdaptiveAllowlistDraftStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_adaptive_allowlist_draft("code-138", "label-731");
        assert_eq!(item.status, AdaptiveAllowlistDraftStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_adaptive_allowlist_draft("code-138", "label-731");
        if item.move_to(AdaptiveAllowlistDraftStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveAllowlistDraftStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_adaptive_allowlist_draft("code-138", "label-731");
        item.status = AdaptiveAllowlistDraftStatus::Withdrawn;
        assert!(
            item.move_to(AdaptiveAllowlistDraftStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
