use super::*;

use acme_kit::text;

/// Where a fallback postmortem rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackPostmortemRollupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FallbackPostmortemRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FallbackPostmortemRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback postmortem rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemRollup {
    pub code: String,
    pub name: String,
    pub grace_days: i64,
    pub title: String,
    pub expires_at: i64,
    pub tenant_id: String,
    pub tags: Vec<String>,
    pub quantity: i64,
    pub scope: String,
    pub status: FallbackPostmortemRollupStatus,
}

impl FallbackPostmortemRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackPostmortemRollupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackPostmortemRollupStatus) -> bool {
        match self.status {
            FallbackPostmortemRollupStatus::Proposed => {
                match next {
                    FallbackPostmortemRollupStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPostmortemRollupStatus::Approved => {
                match next {
                    FallbackPostmortemRollupStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPostmortemRollupStatus::Rejected => {
                match next {
                    FallbackPostmortemRollupStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FallbackPostmortemRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback postmortem rollup {} cannot move from {} to {}",
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

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_value(&self) -> f64 {
        ((self.quantity as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackPostmortemRollup {
        FallbackPostmortemRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            grace_days: self.grace_days,
            title: self.title.clone(),
            expires_at: self.expires_at,
            tenant_id: self.tenant_id.clone(),
            tags: self.tags.clone(),
            quantity: self.quantity,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a fallback postmortem rollup in status proposed.
pub fn new_fallback_postmortem_rollup(code: &str, name: &str) -> FallbackPostmortemRollup {
    FallbackPostmortemRollup {
        code: code.to_string(),
        name: name.to_string(),
        grace_days: 0,
        title: "".to_string(),
        expires_at: 0,
        tenant_id: "".to_string(),
        tags: Vec::new(),
        quantity: 0,
        scope: "".to_string(),
        status: FallbackPostmortemRollupStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_fallback_postmortem_rollup("code-176", "name-738");
        assert_eq!(item.status, FallbackPostmortemRollupStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_fallback_postmortem_rollup("code-176", "name-738");
        if item.move_to(FallbackPostmortemRollupStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackPostmortemRollupStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_fallback_postmortem_rollup("code-176", "name-738");
        item.status = FallbackPostmortemRollupStatus::Withdrawn;
        assert!(
            item.move_to(FallbackPostmortemRollupStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
