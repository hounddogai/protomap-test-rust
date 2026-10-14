use super::*;

use acme_kit::text;

/// Where a probe baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeBaselineStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ProbeBaselineStatus {
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

impl std::fmt::Display for ProbeBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One probe baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeBaseline {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub expires_at: i64,
    pub tags: Vec<String>,
    pub reviewer: String,
    pub cluster: String,
    pub description: String,
    pub status: ProbeBaselineStatus,
}

impl ProbeBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeBaselineStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeBaselineStatus) -> bool {
        match self.status {
            ProbeBaselineStatus::New => {
                return next == ProbeBaselineStatus::Done;
            }
            ProbeBaselineStatus::Queued => {
                match next {
                    ProbeBaselineStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeBaselineStatus::Running => {
                match next {
                    ProbeBaselineStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeBaselineStatus::Done => {
                return next == ProbeBaselineStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ProbeBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe baseline {} cannot move from {} to {}",
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
        format!("{} {}", "probe baseline", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.expires_at as f64
    }

    pub fn copy(&self) -> ProbeBaseline {
        ProbeBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            expires_at: self.expires_at,
            tags: self.tags.clone(),
            reviewer: self.reviewer.clone(),
            cluster: self.cluster.clone(),
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a probe baseline in status new.
pub fn new_probe_baseline(code: &str, title: &str) -> ProbeBaseline {
    ProbeBaseline {
        code: code.to_string(),
        title: title.to_string(),
        owner_id: "".to_string(),
        expires_at: 0,
        tags: Vec::new(),
        reviewer: "".to_string(),
        cluster: "".to_string(),
        description: "".to_string(),
        status: ProbeBaselineStatus::New,
    }
}
