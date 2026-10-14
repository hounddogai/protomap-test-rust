use super::*;

use acme_kit::text;

/// Where a ttl policy target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyTargetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TtlPolicyTargetStatus {
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

impl std::fmt::Display for TtlPolicyTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One ttl policy target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyTarget {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub external_ref: String,
    pub archived: bool,
    pub quota_units: i64,
    pub score: f64,
    pub revision: i64,
    pub summary: String,
    pub updated_at: i64,
    pub status: TtlPolicyTargetStatus,
}

impl TtlPolicyTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyTargetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyTargetStatus) -> bool {
        match self.status {
            TtlPolicyTargetStatus::Idle => {
                match next {
                    TtlPolicyTargetStatus::Warming | TtlPolicyTargetStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyTargetStatus::Warming => {
                return next == TtlPolicyTargetStatus::Ready;
            }
            TtlPolicyTargetStatus::Ready => {
                return next == TtlPolicyTargetStatus::Draining;
            }
            TtlPolicyTargetStatus::Draining => {
                return next == TtlPolicyTargetStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.revision as f64) * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyTarget {
        TtlPolicyTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            external_ref: self.external_ref.clone(),
            archived: self.archived,
            quota_units: self.quota_units,
            score: self.score,
            revision: self.revision,
            summary: self.summary.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a ttl policy target in status idle.
pub fn new_ttl_policy_target(code: &str, label: &str) -> TtlPolicyTarget {
    TtlPolicyTarget {
        code: code.to_string(),
        label: label.to_string(),
        locale: "".to_string(),
        external_ref: "".to_string(),
        archived: false,
        quota_units: 0,
        score: 0.0,
        revision: 0,
        summary: "".to_string(),
        updated_at: 0,
        status: TtlPolicyTargetStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_ttl_policy_target("code-512", "label-308");
        assert_eq!(item.status, TtlPolicyTargetStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_ttl_policy_target("code-512", "label-308");
        if item.move_to(TtlPolicyTargetStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyTargetStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_ttl_policy_target("code-512", "label-308");
        item.status = TtlPolicyTargetStatus::Stopped;
        assert!(
            item.move_to(TtlPolicyTargetStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
