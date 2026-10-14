use super::*;

use acme_kit::text;

/// Where a release claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseClaimStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ReleaseClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for ReleaseClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A release claim as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseClaim {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub visible: bool,
    pub locale: String,
    pub segment: String,
    pub starts_at: i64,
    pub expires_at: i64,
    pub phone: String,
    pub status: ReleaseClaimStatus,
}

impl ReleaseClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseClaimStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseClaimStatus) -> bool {
        match self.status {
            ReleaseClaimStatus::Planned => {
                return next == ReleaseClaimStatus::InProgress || next == ReleaseClaimStatus::Cancelled;
            }
            ReleaseClaimStatus::Scheduled => {
                return next == ReleaseClaimStatus::Completed;
            }
            ReleaseClaimStatus::InProgress => {
                return next == ReleaseClaimStatus::Completed || next == ReleaseClaimStatus::Cancelled;
            }
            ReleaseClaimStatus::Completed => {
                return next == ReleaseClaimStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleaseClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.starts_at as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseClaim {
        ReleaseClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            visible: self.visible,
            locale: self.locale.clone(),
            segment: self.segment.clone(),
            starts_at: self.starts_at,
            expires_at: self.expires_at,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a release claim in status planned.
pub fn new_release_claim(code: &str, label: &str) -> ReleaseClaim {
    ReleaseClaim {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        visible: false,
        locale: "".to_string(),
        segment: "".to_string(),
        starts_at: 0,
        expires_at: 0,
        phone: "".to_string(),
        status: ReleaseClaimStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_release_claim("code-405", "label-583");
        assert_eq!(item.status, ReleaseClaimStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_release_claim("code-405", "label-583");
        if item.move_to(ReleaseClaimStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseClaimStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_release_claim("code-405", "label-583");
        item.status = ReleaseClaimStatus::Cancelled;
        assert!(
            item.move_to(ReleaseClaimStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
