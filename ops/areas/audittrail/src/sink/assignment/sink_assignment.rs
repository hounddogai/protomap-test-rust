use super::*;

use acme_kit::text;

/// Where a sink assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkAssignmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SinkAssignmentStatus {
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

impl std::fmt::Display for SinkAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One sink assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkAssignment {
    pub code: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub unit: String,
    pub limit: i64,
    pub port: i64,
    pub retries: i64,
    pub rate: f64,
    pub version: i64,
    pub channel: String,
    pub amount: f64,
    pub status: SinkAssignmentStatus,
}

impl SinkAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkAssignmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkAssignmentStatus) -> bool {
        match self.status {
            SinkAssignmentStatus::New => {
                match next {
                    SinkAssignmentStatus::Queued | SinkAssignmentStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkAssignmentStatus::Queued => {
                match next {
                    SinkAssignmentStatus::Running | SinkAssignmentStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkAssignmentStatus::Running => {
                match next {
                    SinkAssignmentStatus::Done | SinkAssignmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkAssignmentStatus::Done => {
                match next {
                    SinkAssignmentStatus::Failed => {
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

    pub fn move_to(&mut self, next: SinkAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.limit as f64 - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkAssignment {
        SinkAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            unit: self.unit.clone(),
            limit: self.limit,
            port: self.port,
            retries: self.retries,
            rate: self.rate,
            version: self.version,
            channel: self.channel.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a sink assignment in status new.
pub fn new_sink_assignment(code: &str, label: &str) -> SinkAssignment {
    SinkAssignment {
        code: code.to_string(),
        label: label.to_string(),
        ttl_seconds: 0,
        unit: "".to_string(),
        limit: 0,
        port: 0,
        retries: 0,
        rate: 0.0,
        version: 0,
        channel: "".to_string(),
        amount: 0.0,
        status: SinkAssignmentStatus::New,
    }
}
