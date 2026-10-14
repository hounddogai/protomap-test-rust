use super::*;

use acme_kit::text;

/// Where a manual verifier notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualVerifierNoticeStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ManualVerifierNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for ManualVerifierNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual verifier notice as the verifier team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierNotice {
    pub code: String,
    pub title: String,
    pub contact_email: String,
    pub starts_at: i64,
    pub score: f64,
    pub segment: String,
    pub rate: f64,
    pub summary: String,
    pub floor_value: f64,
    pub weight: f64,
    pub approved: bool,
    pub status: ManualVerifierNoticeStatus,
}

impl ManualVerifierNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualVerifierNoticeStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualVerifierNoticeStatus) -> bool {
        match self.status {
            ManualVerifierNoticeStatus::New => {
                match next {
                    ManualVerifierNoticeStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualVerifierNoticeStatus::Queued => {
                match next {
                    ManualVerifierNoticeStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualVerifierNoticeStatus::Running => {
                return next == ManualVerifierNoticeStatus::Done || next == ManualVerifierNoticeStatus::Failed;
            }
            ManualVerifierNoticeStatus::Done => {
                match next {
                    ManualVerifierNoticeStatus::Failed => {
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

    pub fn move_to(&mut self, next: ManualVerifierNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual verifier notice {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "manual verifier notice", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.score - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualVerifierNotice {
        ManualVerifierNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            starts_at: self.starts_at,
            score: self.score,
            segment: self.segment.clone(),
            rate: self.rate,
            summary: self.summary.clone(),
            floor_value: self.floor_value,
            weight: self.weight,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a manual verifier notice in status new.
pub fn new_manual_verifier_notice(code: &str, title: &str) -> ManualVerifierNotice {
    ManualVerifierNotice {
        code: code.to_string(),
        title: title.to_string(),
        contact_email: "".to_string(),
        starts_at: 0,
        score: 0.0,
        segment: "".to_string(),
        rate: 0.0,
        summary: "".to_string(),
        floor_value: 0.0,
        weight: 0.0,
        approved: false,
        status: ManualVerifierNoticeStatus::New,
    }
}
