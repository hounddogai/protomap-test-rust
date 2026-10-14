use super::*;

use acme_kit::text;

/// Where a adaptive tenant draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveTenantDraftStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AdaptiveTenantDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AdaptiveTenantDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive tenant draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTenantDraft {
    pub code: String,
    pub name: String,
    pub source: String,
    pub reviewer: String,
    pub expires_at: i64,
    pub capacity: i64,
    pub duration_minutes: i64,
    pub status: AdaptiveTenantDraftStatus,
}

impl AdaptiveTenantDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveTenantDraftStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveTenantDraftStatus) -> bool {
        match self.status {
            AdaptiveTenantDraftStatus::Created => {
                return next == AdaptiveTenantDraftStatus::Published;
            }
            AdaptiveTenantDraftStatus::Validated => {
                return next == AdaptiveTenantDraftStatus::Published || next == AdaptiveTenantDraftStatus::Retired;
            }
            AdaptiveTenantDraftStatus::Published => {
                match next {
                    AdaptiveTenantDraftStatus::Retired => {
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

    pub fn move_to(&mut self, next: AdaptiveTenantDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive tenant draft {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveTenantDraft {
        AdaptiveTenantDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            source: self.source.clone(),
            reviewer: self.reviewer.clone(),
            expires_at: self.expires_at,
            capacity: self.capacity,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a adaptive tenant draft in status created.
pub fn new_adaptive_tenant_draft(code: &str, name: &str) -> AdaptiveTenantDraft {
    AdaptiveTenantDraft {
        code: code.to_string(),
        name: name.to_string(),
        source: "".to_string(),
        reviewer: "".to_string(),
        expires_at: 0,
        capacity: 0,
        duration_minutes: 0,
        status: AdaptiveTenantDraftStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_adaptive_tenant_draft("code-320", "name-395");
        assert_eq!(item.status, AdaptiveTenantDraftStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_adaptive_tenant_draft("code-320", "name-395");
        if item.move_to(AdaptiveTenantDraftStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveTenantDraftStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_adaptive_tenant_draft("code-320", "name-395");
        item.status = AdaptiveTenantDraftStatus::Retired;
        assert!(
            item.move_to(AdaptiveTenantDraftStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
