use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a default query adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultQueryAdjustmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DefaultQueryAdjustmentStatus {
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

impl std::fmt::Display for DefaultQueryAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default query adjustment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQueryAdjustment {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub checksum: String,
    pub reviewer: String,
    pub attributes: BTreeMap<String, String>,
    pub created_at: i64,
    pub issuer: String,
    pub status: DefaultQueryAdjustmentStatus,
}

impl DefaultQueryAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultQueryAdjustmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultQueryAdjustmentStatus) -> bool {
        match self.status {
            DefaultQueryAdjustmentStatus::New => {
                return next == DefaultQueryAdjustmentStatus::Running;
            }
            DefaultQueryAdjustmentStatus::Queued => {
                return next == DefaultQueryAdjustmentStatus::Running || next == DefaultQueryAdjustmentStatus::Failed;
            }
            DefaultQueryAdjustmentStatus::Running => {
                match next {
                    DefaultQueryAdjustmentStatus::Done | DefaultQueryAdjustmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultQueryAdjustmentStatus::Done => {
                return next == DefaultQueryAdjustmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultQueryAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default query adjustment {} cannot move from {} to {}",
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

    pub fn measured_value(&self) -> f64 {
        self.created_at as f64
    }

    pub fn copy(&self) -> DefaultQueryAdjustment {
        DefaultQueryAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            checksum: self.checksum.clone(),
            reviewer: self.reviewer.clone(),
            attributes: self.attributes.clone(),
            created_at: self.created_at,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a default query adjustment in status new.
pub fn new_default_query_adjustment(code: &str, label: &str) -> DefaultQueryAdjustment {
    DefaultQueryAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        checksum: "".to_string(),
        reviewer: "".to_string(),
        attributes: BTreeMap::new(),
        created_at: 0,
        issuer: "".to_string(),
        status: DefaultQueryAdjustmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_default_query_adjustment("code-588", "label-357");
        assert_eq!(item.status, DefaultQueryAdjustmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_default_query_adjustment("code-588", "label-357");
        if item.move_to(DefaultQueryAdjustmentStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultQueryAdjustmentStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_default_query_adjustment("code-588", "label-357");
        item.status = DefaultQueryAdjustmentStatus::Failed;
        assert!(
            item.move_to(DefaultQueryAdjustmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
