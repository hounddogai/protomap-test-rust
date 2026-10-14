use super::*;

use acme_kit::text;

/// Where a strict grant target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictGrantTargetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl StrictGrantTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for StrictGrantTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict grant target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictGrantTarget {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub revision: i64,
    pub limit: i64,
    pub ceiling: f64,
    pub name: String,
    pub starts_at: i64,
    pub archived: bool,
    pub status: StrictGrantTargetStatus,
}

impl StrictGrantTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictGrantTargetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictGrantTargetStatus) -> bool {
        match self.status {
            StrictGrantTargetStatus::Created => {
                return next == StrictGrantTargetStatus::Published;
            }
            StrictGrantTargetStatus::Validated => {
                return next == StrictGrantTargetStatus::Retired;
            }
            StrictGrantTargetStatus::Published => {
                match next {
                    StrictGrantTargetStatus::Retired => {
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

    pub fn move_to(&mut self, next: StrictGrantTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict grant target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.starts_at as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictGrantTarget {
        StrictGrantTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            revision: self.revision,
            limit: self.limit,
            ceiling: self.ceiling,
            name: self.name.clone(),
            starts_at: self.starts_at,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a strict grant target in status created.
pub fn new_strict_grant_target(code: &str, label: &str) -> StrictGrantTarget {
    StrictGrantTarget {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        revision: 0,
        limit: 0,
        ceiling: 0.0,
        name: "".to_string(),
        starts_at: 0,
        archived: false,
        status: StrictGrantTargetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_strict_grant_target("code-377", "label-89");
        assert_eq!(item.status, StrictGrantTargetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_strict_grant_target("code-377", "label-89");
        if item.move_to(StrictGrantTargetStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictGrantTargetStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_strict_grant_target("code-377", "label-89");
        item.status = StrictGrantTargetStatus::Retired;
        assert!(
            item.move_to(StrictGrantTargetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
