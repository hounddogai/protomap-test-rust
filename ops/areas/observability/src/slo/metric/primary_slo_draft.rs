use super::*;

use acme_kit::text;

/// Where a primary slo draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimarySloDraftStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PrimarySloDraftStatus {
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

impl std::fmt::Display for PrimarySloDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary slo draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloDraft {
    pub code: String,
    pub title: String,
    pub starts_at: i64,
    pub key_id: String,
    pub visible: bool,
    pub weight: f64,
    pub description: String,
    pub owner_id: String,
    pub margin: f64,
    pub notes: String,
    pub status: PrimarySloDraftStatus,
}

impl PrimarySloDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimarySloDraftStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimarySloDraftStatus) -> bool {
        match self.status {
            PrimarySloDraftStatus::Idle => {
                match next {
                    PrimarySloDraftStatus::Ready | PrimarySloDraftStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimarySloDraftStatus::Warming => {
                match next {
                    PrimarySloDraftStatus::Draining | PrimarySloDraftStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimarySloDraftStatus::Ready => {
                match next {
                    PrimarySloDraftStatus::Draining | PrimarySloDraftStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimarySloDraftStatus::Draining => {
                match next {
                    PrimarySloDraftStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PrimarySloDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary slo draft {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.starts_at as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimarySloDraft {
        PrimarySloDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            starts_at: self.starts_at,
            key_id: self.key_id.clone(),
            visible: self.visible,
            weight: self.weight,
            description: self.description.clone(),
            owner_id: self.owner_id.clone(),
            margin: self.margin,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary slo draft in status idle.
pub fn new_primary_slo_draft(code: &str, title: &str) -> PrimarySloDraft {
    PrimarySloDraft {
        code: code.to_string(),
        title: title.to_string(),
        starts_at: 0,
        key_id: "".to_string(),
        visible: false,
        weight: 0.0,
        description: "".to_string(),
        owner_id: "".to_string(),
        margin: 0.0,
        notes: "".to_string(),
        status: PrimarySloDraftStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_primary_slo_draft("code-557", "title-883");
        assert_eq!(item.status, PrimarySloDraftStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_primary_slo_draft("code-557", "title-883");
        if item.move_to(PrimarySloDraftStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimarySloDraftStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_primary_slo_draft("code-557", "title-883");
        item.status = PrimarySloDraftStatus::Stopped;
        assert!(
            item.move_to(PrimarySloDraftStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
