use super::*;

use acme_kit::text;

/// Where a regional failover checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalFailoverCheckpointStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionalFailoverCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionalFailoverCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional failover checkpoint as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverCheckpoint {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub country: String,
    pub checksum: String,
    pub source: String,
    pub status: RegionalFailoverCheckpointStatus,
}

impl RegionalFailoverCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalFailoverCheckpointStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalFailoverCheckpointStatus) -> bool {
        match self.status {
            RegionalFailoverCheckpointStatus::Created => {
                match next {
                    RegionalFailoverCheckpointStatus::Published | RegionalFailoverCheckpointStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalFailoverCheckpointStatus::Validated => {
                match next {
                    RegionalFailoverCheckpointStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalFailoverCheckpointStatus::Published => {
                match next {
                    RegionalFailoverCheckpointStatus::Retired => {
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

    pub fn move_to(&mut self, next: RegionalFailoverCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional failover checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional failover checkpoint", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> RegionalFailoverCheckpoint {
        RegionalFailoverCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            country: self.country.clone(),
            checksum: self.checksum.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a regional failover checkpoint in status created.
pub fn new_regional_failover_checkpoint(code: &str, label: &str) -> RegionalFailoverCheckpoint {
    RegionalFailoverCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        country: "".to_string(),
        checksum: "".to_string(),
        source: "".to_string(),
        status: RegionalFailoverCheckpointStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_regional_failover_checkpoint("code-672", "label-199");
        assert_eq!(item.status, RegionalFailoverCheckpointStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_regional_failover_checkpoint("code-672", "label-199");
        if item.move_to(RegionalFailoverCheckpointStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalFailoverCheckpointStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_regional_failover_checkpoint("code-672", "label-199");
        item.status = RegionalFailoverCheckpointStatus::Retired;
        assert!(
            item.move_to(RegionalFailoverCheckpointStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
