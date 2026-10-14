use super::*;

use acme_kit::text;

/// Where a probe tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeTierStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ProbeTierStatus {
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

impl std::fmt::Display for ProbeTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a probe tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeTier {
    pub code: String,
    pub title: String,
    pub quota_units: i64,
    pub version: i64,
    pub audience: String,
    pub fingerprint: String,
    pub status: ProbeTierStatus,
}

impl ProbeTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeTierStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeTierStatus) -> bool {
        match self.status {
            ProbeTierStatus::New => {
                match next {
                    ProbeTierStatus::Queued | ProbeTierStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeTierStatus::Queued => {
                match next {
                    ProbeTierStatus::Running | ProbeTierStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeTierStatus::Running => {
                match next {
                    ProbeTierStatus::Done | ProbeTierStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeTierStatus::Done => {
                match next {
                    ProbeTierStatus::Failed => {
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

    pub fn move_to(&mut self, next: ProbeTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.quota_units as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeTier {
        ProbeTier {
            code: self.code.clone(),
            title: self.title.clone(),
            quota_units: self.quota_units,
            version: self.version,
            audience: self.audience.clone(),
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a probe tier in status new.
pub fn new_probe_tier(code: &str, title: &str) -> ProbeTier {
    ProbeTier {
        code: code.to_string(),
        title: title.to_string(),
        quota_units: 0,
        version: 0,
        audience: "".to_string(),
        fingerprint: "".to_string(),
        status: ProbeTierStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_probe_tier("code-767", "title-756");
        assert_eq!(item.status, ProbeTierStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_probe_tier("code-767", "title-756");
        if item.move_to(ProbeTierStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeTierStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_probe_tier("code-767", "title-756");
        item.status = ProbeTierStatus::Failed;
        assert!(item.move_to(ProbeTierStatus::New).is_err(), "expected move to fail");
    }
}
