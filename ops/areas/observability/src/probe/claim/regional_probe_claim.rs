use super::*;

use acme_kit::text;

/// Where a regional probe claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalProbeClaimStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RegionalProbeClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for RegionalProbeClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional probe claim. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeClaim {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub duration_minutes: i64,
    pub notes: String,
    pub principal: String,
    pub expires_at: i64,
    pub visible: bool,
    pub external_ref: String,
    pub status: RegionalProbeClaimStatus,
}

impl RegionalProbeClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalProbeClaimStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalProbeClaimStatus) -> bool {
        match self.status {
            RegionalProbeClaimStatus::Open => {
                match next {
                    RegionalProbeClaimStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalProbeClaimStatus::InReview => {
                match next {
                    RegionalProbeClaimStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalProbeClaimStatus::Resolved => {
                return next == RegionalProbeClaimStatus::Reopened || next == RegionalProbeClaimStatus::Archived;
            }
            RegionalProbeClaimStatus::Reopened => {
                return next == RegionalProbeClaimStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalProbeClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional probe claim {} cannot move from {} to {}",
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
        format!("{} {}", "regional probe claim", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.expires_at as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalProbeClaim {
        RegionalProbeClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            duration_minutes: self.duration_minutes,
            notes: self.notes.clone(),
            principal: self.principal.clone(),
            expires_at: self.expires_at,
            visible: self.visible,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional probe claim in status open.
pub fn new_regional_probe_claim(code: &str, label: &str) -> RegionalProbeClaim {
    RegionalProbeClaim {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        duration_minutes: 0,
        notes: "".to_string(),
        principal: "".to_string(),
        expires_at: 0,
        visible: false,
        external_ref: "".to_string(),
        status: RegionalProbeClaimStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_regional_probe_claim("code-497", "label-46");
        assert_eq!(item.status, RegionalProbeClaimStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_regional_probe_claim("code-497", "label-46");
        if item.move_to(RegionalProbeClaimStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalProbeClaimStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_regional_probe_claim("code-497", "label-46");
        item.status = RegionalProbeClaimStatus::Archived;
        assert!(
            item.move_to(RegionalProbeClaimStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
