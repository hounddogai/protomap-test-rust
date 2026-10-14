use super::*;

use acme_kit::text;

/// Where a ttl policy manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyManifestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TtlPolicyManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for TtlPolicyManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One ttl policy manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyManifest {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub locale: String,
    pub reviewer: String,
    pub weight: f64,
    pub scope: String,
    pub summary: String,
    pub margin: f64,
    pub duration_minutes: i64,
    pub segment: String,
    pub status: TtlPolicyManifestStatus,
}

impl TtlPolicyManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyManifestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyManifestStatus) -> bool {
        match self.status {
            TtlPolicyManifestStatus::Draft => {
                return next == TtlPolicyManifestStatus::Pending || next == TtlPolicyManifestStatus::Suspended;
            }
            TtlPolicyManifestStatus::Pending => {
                return next == TtlPolicyManifestStatus::Active || next == TtlPolicyManifestStatus::Closed;
            }
            TtlPolicyManifestStatus::Active => {
                return next == TtlPolicyManifestStatus::Suspended || next == TtlPolicyManifestStatus::Closed;
            }
            TtlPolicyManifestStatus::Suspended => {
                return next == TtlPolicyManifestStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.duration_minutes as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyManifest {
        TtlPolicyManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            locale: self.locale.clone(),
            reviewer: self.reviewer.clone(),
            weight: self.weight,
            scope: self.scope.clone(),
            summary: self.summary.clone(),
            margin: self.margin,
            duration_minutes: self.duration_minutes,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a ttl policy manifest in status draft.
pub fn new_ttl_policy_manifest(code: &str, title: &str) -> TtlPolicyManifest {
    TtlPolicyManifest {
        code: code.to_string(),
        title: title.to_string(),
        fingerprint: "".to_string(),
        locale: "".to_string(),
        reviewer: "".to_string(),
        weight: 0.0,
        scope: "".to_string(),
        summary: "".to_string(),
        margin: 0.0,
        duration_minutes: 0,
        segment: "".to_string(),
        status: TtlPolicyManifestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_ttl_policy_manifest("code-94", "title-88");
        assert_eq!(item.status, TtlPolicyManifestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_ttl_policy_manifest("code-94", "title-88");
        if item.move_to(TtlPolicyManifestStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyManifestStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_ttl_policy_manifest("code-94", "title-88");
        item.status = TtlPolicyManifestStatus::Closed;
        assert!(
            item.move_to(TtlPolicyManifestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
