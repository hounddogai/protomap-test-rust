use super::*;

use acme_kit::text;

/// Where a probe digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeDigestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ProbeDigestStatus {
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

impl std::fmt::Display for ProbeDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A probe digest as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeDigest {
    pub code: String,
    pub name: String,
    pub category: String,
    pub grace_days: i64,
    pub budget: f64,
    pub status: ProbeDigestStatus,
}

impl ProbeDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeDigestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeDigestStatus) -> bool {
        match self.status {
            ProbeDigestStatus::New => {
                match next {
                    ProbeDigestStatus::Queued | ProbeDigestStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeDigestStatus::Queued => {
                match next {
                    ProbeDigestStatus::Running | ProbeDigestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeDigestStatus::Running => {
                match next {
                    ProbeDigestStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeDigestStatus::Done => {
                return next == ProbeDigestStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ProbeDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.grace_days as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeDigest {
        ProbeDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            grace_days: self.grace_days,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a probe digest in status new.
pub fn new_probe_digest(code: &str, name: &str) -> ProbeDigest {
    ProbeDigest {
        code: code.to_string(),
        name: name.to_string(),
        category: "".to_string(),
        grace_days: 0,
        budget: 0.0,
        status: ProbeDigestStatus::New,
    }
}
