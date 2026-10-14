use super::*;

use acme_kit::text;

/// Where a grant baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantBaselineStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GrantBaselineStatus {
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

impl std::fmt::Display for GrantBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One grant baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantBaseline {
    pub code: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub contact_email: String,
    pub segment: String,
    pub limit: i64,
    pub expires_at: i64,
    pub score: f64,
    pub port: i64,
    pub rotation_days: i64,
    pub summary: String,
    pub status: GrantBaselineStatus,
}

impl GrantBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantBaselineStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantBaselineStatus) -> bool {
        match self.status {
            GrantBaselineStatus::Planned => {
                match next {
                    GrantBaselineStatus::InProgress | GrantBaselineStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantBaselineStatus::Scheduled => {
                match next {
                    GrantBaselineStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantBaselineStatus::InProgress => {
                return next == GrantBaselineStatus::Completed || next == GrantBaselineStatus::Cancelled;
            }
            GrantBaselineStatus::Completed => {
                return next == GrantBaselineStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GrantBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant baseline {} cannot move from {} to {}",
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
        format!("{} {}", "grant baseline", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantBaseline {
        GrantBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            contact_email: self.contact_email.clone(),
            segment: self.segment.clone(),
            limit: self.limit,
            expires_at: self.expires_at,
            score: self.score,
            port: self.port,
            rotation_days: self.rotation_days,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a grant baseline in status planned.
pub fn new_grant_baseline(code: &str, label: &str) -> GrantBaseline {
    GrantBaseline {
        code: code.to_string(),
        label: label.to_string(),
        ttl_seconds: 0,
        contact_email: "".to_string(),
        segment: "".to_string(),
        limit: 0,
        expires_at: 0,
        score: 0.0,
        port: 0,
        rotation_days: 0,
        summary: "".to_string(),
        status: GrantBaselineStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_grant_baseline("code-468", "label-755");
        assert_eq!(item.status, GrantBaselineStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_grant_baseline("code-468", "label-755");
        if item.move_to(GrantBaselineStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantBaselineStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_grant_baseline("code-468", "label-755");
        item.status = GrantBaselineStatus::Cancelled;
        assert!(
            item.move_to(GrantBaselineStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
