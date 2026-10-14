use super::*;

use acme_kit::text;

/// Where a tiered approval preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredApprovalPreferenceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TieredApprovalPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TieredApprovalPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered approval preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalPreference {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub issuer: String,
    pub channel: String,
    pub rotation_days: i64,
    pub updated_at: i64,
    pub score: f64,
    pub scope: String,
    pub margin: f64,
    pub status: TieredApprovalPreferenceStatus,
}

impl TieredApprovalPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredApprovalPreferenceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredApprovalPreferenceStatus) -> bool {
        match self.status {
            TieredApprovalPreferenceStatus::Proposed => {
                return next == TieredApprovalPreferenceStatus::Rejected;
            }
            TieredApprovalPreferenceStatus::Approved => {
                return next == TieredApprovalPreferenceStatus::Withdrawn;
            }
            TieredApprovalPreferenceStatus::Rejected => {
                match next {
                    TieredApprovalPreferenceStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: TieredApprovalPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered approval preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered approval preference", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.rate - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredApprovalPreference {
        TieredApprovalPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            issuer: self.issuer.clone(),
            channel: self.channel.clone(),
            rotation_days: self.rotation_days,
            updated_at: self.updated_at,
            score: self.score,
            scope: self.scope.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a tiered approval preference in status proposed.
pub fn new_tiered_approval_preference(code: &str, title: &str) -> TieredApprovalPreference {
    TieredApprovalPreference {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        issuer: "".to_string(),
        channel: "".to_string(),
        rotation_days: 0,
        updated_at: 0,
        score: 0.0,
        scope: "".to_string(),
        margin: 0.0,
        status: TieredApprovalPreferenceStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_tiered_approval_preference("code-135", "title-643");
        assert_eq!(item.status, TieredApprovalPreferenceStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_tiered_approval_preference("code-135", "title-643");
        if item.move_to(TieredApprovalPreferenceStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredApprovalPreferenceStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_tiered_approval_preference("code-135", "title-643");
        item.status = TieredApprovalPreferenceStatus::Withdrawn;
        assert!(
            item.move_to(TieredApprovalPreferenceStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
