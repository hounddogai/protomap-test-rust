use super::*;

use acme_kit::text;

/// Where a zone checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneCheckpointStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ZoneCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ZoneCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One zone checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneCheckpoint {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub archived: bool,
    pub score: f64,
    pub sequence: i64,
    pub revision: i64,
    pub active: bool,
    pub ends_at: i64,
    pub status: ZoneCheckpointStatus,
}

impl ZoneCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneCheckpointStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneCheckpointStatus) -> bool {
        match self.status {
            ZoneCheckpointStatus::Created => {
                return next == ZoneCheckpointStatus::Published || next == ZoneCheckpointStatus::Retired;
            }
            ZoneCheckpointStatus::Validated => {
                match next {
                    ZoneCheckpointStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneCheckpointStatus::Published => {
                return next == ZoneCheckpointStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ZoneCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "zone checkpoint", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.score + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneCheckpoint {
        ZoneCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            archived: self.archived,
            score: self.score,
            sequence: self.sequence,
            revision: self.revision,
            active: self.active,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a zone checkpoint in status created.
pub fn new_zone_checkpoint(code: &str, title: &str) -> ZoneCheckpoint {
    ZoneCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        archived: false,
        score: 0.0,
        sequence: 0,
        revision: 0,
        active: false,
        ends_at: 0,
        status: ZoneCheckpointStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_zone_checkpoint("code-183", "title-803");
        assert_eq!(item.status, ZoneCheckpointStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_zone_checkpoint("code-183", "title-803");
        if item.move_to(ZoneCheckpointStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneCheckpointStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_zone_checkpoint("code-183", "title-803");
        item.status = ZoneCheckpointStatus::Retired;
        assert!(
            item.move_to(ZoneCheckpointStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
