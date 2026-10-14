use super::*;

use acme_kit::text;

/// Where a legacy silence window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacySilenceWindowStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacySilenceWindowStatus {
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

impl std::fmt::Display for LegacySilenceWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy silence window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceWindow {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub threshold: f64,
    pub name: String,
    pub budget: f64,
    pub archived: bool,
    pub rotation_days: i64,
    pub starts_at: i64,
    pub cluster: String,
    pub status: LegacySilenceWindowStatus,
}

impl LegacySilenceWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacySilenceWindowStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacySilenceWindowStatus) -> bool {
        match self.status {
            LegacySilenceWindowStatus::Open => {
                match next {
                    LegacySilenceWindowStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacySilenceWindowStatus::InReview => {
                match next {
                    LegacySilenceWindowStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacySilenceWindowStatus::Resolved => {
                return next == LegacySilenceWindowStatus::Reopened;
            }
            LegacySilenceWindowStatus::Reopened => {
                return next == LegacySilenceWindowStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacySilenceWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy silence window {} cannot move from {} to {}",
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
        format!("{} {}", "legacy silence window", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.threshold + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacySilenceWindow {
        LegacySilenceWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            threshold: self.threshold,
            name: self.name.clone(),
            budget: self.budget,
            archived: self.archived,
            rotation_days: self.rotation_days,
            starts_at: self.starts_at,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a legacy silence window in status open.
pub fn new_legacy_silence_window(code: &str, title: &str) -> LegacySilenceWindow {
    LegacySilenceWindow {
        code: code.to_string(),
        title: title.to_string(),
        fingerprint: "".to_string(),
        threshold: 0.0,
        name: "".to_string(),
        budget: 0.0,
        archived: false,
        rotation_days: 0,
        starts_at: 0,
        cluster: "".to_string(),
        status: LegacySilenceWindowStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_legacy_silence_window("code-572", "title-648");
        assert_eq!(item.status, LegacySilenceWindowStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_legacy_silence_window("code-572", "title-648");
        if item.move_to(LegacySilenceWindowStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacySilenceWindowStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_legacy_silence_window("code-572", "title-648");
        item.status = LegacySilenceWindowStatus::Archived;
        assert!(
            item.move_to(LegacySilenceWindowStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
