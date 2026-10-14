use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a rolling usage checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingUsageCheckpointStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollingUsageCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RollingUsageCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling usage checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingUsageCheckpoint {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub ceiling: f64,
    pub description: String,
    pub weight: f64,
    pub channel: String,
    pub cipher: String,
    pub category: String,
    pub active: bool,
    pub host: String,
    pub status: RollingUsageCheckpointStatus,
}

impl RollingUsageCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingUsageCheckpointStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingUsageCheckpointStatus) -> bool {
        match self.status {
            RollingUsageCheckpointStatus::Draft => {
                match next {
                    RollingUsageCheckpointStatus::Suspended | RollingUsageCheckpointStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingUsageCheckpointStatus::Pending => {
                match next {
                    RollingUsageCheckpointStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingUsageCheckpointStatus::Active => {
                match next {
                    RollingUsageCheckpointStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingUsageCheckpointStatus::Suspended => {
                match next {
                    RollingUsageCheckpointStatus::Closed => {
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

    pub fn move_to(&mut self, next: RollingUsageCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling usage checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn effective_value(&self) -> f64 {
        ((self.weight - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingUsageCheckpoint {
        RollingUsageCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            ceiling: self.ceiling,
            description: self.description.clone(),
            weight: self.weight,
            channel: self.channel.clone(),
            cipher: self.cipher.clone(),
            category: self.category.clone(),
            active: self.active,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a rolling usage checkpoint in status draft.
pub fn new_rolling_usage_checkpoint(code: &str, label: &str) -> RollingUsageCheckpoint {
    RollingUsageCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        ceiling: 0.0,
        description: "".to_string(),
        weight: 0.0,
        channel: "".to_string(),
        cipher: "".to_string(),
        category: "".to_string(),
        active: false,
        host: "".to_string(),
        status: RollingUsageCheckpointStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rolling_usage_checkpoint("code-144", "label-897");
        assert_eq!(item.status, RollingUsageCheckpointStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_rolling_usage_checkpoint("code-144", "label-897");
        if item.move_to(RollingUsageCheckpointStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingUsageCheckpointStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rolling_usage_checkpoint("code-144", "label-897");
        item.status = RollingUsageCheckpointStatus::Closed;
        assert!(
            item.move_to(RollingUsageCheckpointStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
