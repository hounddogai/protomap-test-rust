use super::*;

use acme_kit::text;

/// Where a overage mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageMappingStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl OverageMappingStatus {
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

impl std::fmt::Display for OverageMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One overage mapping, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageMapping {
    pub code: String,
    pub name: String,
    pub description: String,
    pub port: i64,
    pub host: String,
    pub created_at: i64,
    pub status: OverageMappingStatus,
}

impl OverageMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageMappingStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageMappingStatus) -> bool {
        match self.status {
            OverageMappingStatus::New => {
                return next == OverageMappingStatus::Running;
            }
            OverageMappingStatus::Queued => {
                return next == OverageMappingStatus::Done;
            }
            OverageMappingStatus::Running => {
                match next {
                    OverageMappingStatus::Done | OverageMappingStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageMappingStatus::Done => {
                match next {
                    OverageMappingStatus::Failed => {
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

    pub fn move_to(&mut self, next: OverageMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage mapping {} cannot move from {} to {}",
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
        format!("{} {}", "overage mapping", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.port as f64 - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageMapping {
        OverageMapping {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            port: self.port,
            host: self.host.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a overage mapping in status new.
pub fn new_overage_mapping(code: &str, name: &str) -> OverageMapping {
    OverageMapping {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        port: 0,
        host: "".to_string(),
        created_at: 0,
        status: OverageMappingStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_overage_mapping("code-875", "name-754");
        assert_eq!(item.status, OverageMappingStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_overage_mapping("code-875", "name-754");
        if item.move_to(OverageMappingStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageMappingStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_overage_mapping("code-875", "name-754");
        item.status = OverageMappingStatus::Failed;
        assert!(
            item.move_to(OverageMappingStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
