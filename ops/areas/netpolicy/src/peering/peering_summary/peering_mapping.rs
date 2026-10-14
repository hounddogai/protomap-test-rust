use super::*;

use acme_kit::text;

/// Where a peering mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringMappingStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PeeringMappingStatus {
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

impl std::fmt::Display for PeeringMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One peering mapping, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringMapping {
    pub code: String,
    pub title: String,
    pub unit: String,
    pub batch_size: i64,
    pub description: String,
    pub visible: bool,
    pub key_id: String,
    pub created_at: i64,
    pub status: PeeringMappingStatus,
}

impl PeeringMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringMappingStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringMappingStatus) -> bool {
        match self.status {
            PeeringMappingStatus::New => {
                return next == PeeringMappingStatus::Done || next == PeeringMappingStatus::Failed;
            }
            PeeringMappingStatus::Queued => {
                match next {
                    PeeringMappingStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringMappingStatus::Running => {
                return next == PeeringMappingStatus::Done;
            }
            PeeringMappingStatus::Done => {
                return next == PeeringMappingStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering mapping {} cannot move from {} to {}",
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
        format!("{} {}", "peering mapping", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.batch_size as f64 - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringMapping {
        PeeringMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            unit: self.unit.clone(),
            batch_size: self.batch_size,
            description: self.description.clone(),
            visible: self.visible,
            key_id: self.key_id.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a peering mapping in status new.
pub fn new_peering_mapping(code: &str, title: &str) -> PeeringMapping {
    PeeringMapping {
        code: code.to_string(),
        title: title.to_string(),
        unit: "".to_string(),
        batch_size: 0,
        description: "".to_string(),
        visible: false,
        key_id: "".to_string(),
        created_at: 0,
        status: PeeringMappingStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_peering_mapping("code-395", "title-746");
        assert_eq!(item.status, PeeringMappingStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_peering_mapping("code-395", "title-746");
        if item.move_to(PeeringMappingStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringMappingStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_peering_mapping("code-395", "title-746");
        item.status = PeeringMappingStatus::Failed;
        assert!(
            item.move_to(PeeringMappingStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
