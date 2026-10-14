use super::*;

use acme_kit::text;

/// Where a tiered plan draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredPlanDraftStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TieredPlanDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TieredPlanDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered plan draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanDraft {
    pub code: String,
    pub label: String,
    pub region: String,
    pub principal: String,
    pub duration_minutes: i64,
    pub grace_days: i64,
    pub maximum: f64,
    pub ttl_seconds: i64,
    pub cluster: String,
    pub audience: String,
    pub status: TieredPlanDraftStatus,
}

impl TieredPlanDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredPlanDraftStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredPlanDraftStatus) -> bool {
        match self.status {
            TieredPlanDraftStatus::Requested => {
                return next == TieredPlanDraftStatus::Revoked;
            }
            TieredPlanDraftStatus::Granted => {
                return next == TieredPlanDraftStatus::Expired;
            }
            TieredPlanDraftStatus::Revoked => {
                return next == TieredPlanDraftStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredPlanDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered plan draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredPlanDraft {
        TieredPlanDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            principal: self.principal.clone(),
            duration_minutes: self.duration_minutes,
            grace_days: self.grace_days,
            maximum: self.maximum,
            ttl_seconds: self.ttl_seconds,
            cluster: self.cluster.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a tiered plan draft in status requested.
pub fn new_tiered_plan_draft(code: &str, label: &str) -> TieredPlanDraft {
    TieredPlanDraft {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        principal: "".to_string(),
        duration_minutes: 0,
        grace_days: 0,
        maximum: 0.0,
        ttl_seconds: 0,
        cluster: "".to_string(),
        audience: "".to_string(),
        status: TieredPlanDraftStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_tiered_plan_draft("code-540", "label-171");
        assert_eq!(item.status, TieredPlanDraftStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_tiered_plan_draft("code-540", "label-171");
        if item.move_to(TieredPlanDraftStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredPlanDraftStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_tiered_plan_draft("code-540", "label-171");
        item.status = TieredPlanDraftStatus::Expired;
        assert!(
            item.move_to(TieredPlanDraftStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
