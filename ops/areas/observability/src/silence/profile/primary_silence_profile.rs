use super::*;

use acme_kit::text;

/// Where a primary silence profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimarySilenceProfileStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PrimarySilenceProfileStatus {
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

impl std::fmt::Display for PrimarySilenceProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary silence profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySilenceProfile {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub summary: String,
    pub threshold: f64,
    pub segment: String,
    pub status: PrimarySilenceProfileStatus,
}

impl PrimarySilenceProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimarySilenceProfileStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimarySilenceProfileStatus) -> bool {
        match self.status {
            PrimarySilenceProfileStatus::New => {
                return next == PrimarySilenceProfileStatus::Running || next == PrimarySilenceProfileStatus::Failed;
            }
            PrimarySilenceProfileStatus::Queued => {
                match next {
                    PrimarySilenceProfileStatus::Done | PrimarySilenceProfileStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimarySilenceProfileStatus::Running => {
                return next == PrimarySilenceProfileStatus::Done || next == PrimarySilenceProfileStatus::Failed;
            }
            PrimarySilenceProfileStatus::Done => {
                match next {
                    PrimarySilenceProfileStatus::Failed => {
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

    pub fn move_to(&mut self, next: PrimarySilenceProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary silence profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.expires_at as f64 - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimarySilenceProfile {
        PrimarySilenceProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            expires_at: self.expires_at,
            summary: self.summary.clone(),
            threshold: self.threshold,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a primary silence profile in status new.
pub fn new_primary_silence_profile(code: &str, name: &str) -> PrimarySilenceProfile {
    PrimarySilenceProfile {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        expires_at: 0,
        summary: "".to_string(),
        threshold: 0.0,
        segment: "".to_string(),
        status: PrimarySilenceProfileStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_primary_silence_profile("code-775", "name-615");
        assert_eq!(item.status, PrimarySilenceProfileStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_primary_silence_profile("code-775", "name-615");
        if item.move_to(PrimarySilenceProfileStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimarySilenceProfileStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_primary_silence_profile("code-775", "name-615");
        item.status = PrimarySilenceProfileStatus::Failed;
        assert!(
            item.move_to(PrimarySilenceProfileStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
