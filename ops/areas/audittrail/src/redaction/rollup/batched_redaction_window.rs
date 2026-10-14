use super::*;

use acme_kit::text;

/// Where a batched redaction window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRedactionWindowStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BatchedRedactionWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BatchedRedactionWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched redaction window as the redaction team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionWindow {
    pub code: String,
    pub name: String,
    pub audience: String,
    pub cluster: String,
    pub duration_minutes: i64,
    pub label: String,
    pub updated_at: i64,
    pub source: String,
    pub sequence: i64,
    pub visible: bool,
    pub status: BatchedRedactionWindowStatus,
}

impl BatchedRedactionWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRedactionWindowStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRedactionWindowStatus) -> bool {
        match self.status {
            BatchedRedactionWindowStatus::Created => {
                return next == BatchedRedactionWindowStatus::Validated
                    || next == BatchedRedactionWindowStatus::Retired;
            }
            BatchedRedactionWindowStatus::Validated => {
                match next {
                    BatchedRedactionWindowStatus::Published | BatchedRedactionWindowStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRedactionWindowStatus::Published => {
                match next {
                    BatchedRedactionWindowStatus::Retired => {
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

    pub fn move_to(&mut self, next: BatchedRedactionWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched redaction window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedRedactionWindow {
        BatchedRedactionWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            audience: self.audience.clone(),
            cluster: self.cluster.clone(),
            duration_minutes: self.duration_minutes,
            label: self.label.clone(),
            updated_at: self.updated_at,
            source: self.source.clone(),
            sequence: self.sequence,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a batched redaction window in status created.
pub fn new_batched_redaction_window(code: &str, name: &str) -> BatchedRedactionWindow {
    BatchedRedactionWindow {
        code: code.to_string(),
        name: name.to_string(),
        audience: "".to_string(),
        cluster: "".to_string(),
        duration_minutes: 0,
        label: "".to_string(),
        updated_at: 0,
        source: "".to_string(),
        sequence: 0,
        visible: false,
        status: BatchedRedactionWindowStatus::Created,
    }
}
