use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a weighted flow log record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedFlowLogRecordStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl WeightedFlowLogRecordStatus {
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

impl std::fmt::Display for WeightedFlowLogRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted flow log record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFlowLogRecord {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub retries: i64,
    pub sequence: i64,
    pub cost: f64,
    pub attributes: BTreeMap<String, String>,
    pub status: WeightedFlowLogRecordStatus,
}

impl WeightedFlowLogRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedFlowLogRecordStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedFlowLogRecordStatus) -> bool {
        match self.status {
            WeightedFlowLogRecordStatus::Idle => {
                return next == WeightedFlowLogRecordStatus::Ready || next == WeightedFlowLogRecordStatus::Draining;
            }
            WeightedFlowLogRecordStatus::Warming => {
                match next {
                    WeightedFlowLogRecordStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedFlowLogRecordStatus::Ready => {
                return next == WeightedFlowLogRecordStatus::Draining;
            }
            WeightedFlowLogRecordStatus::Draining => {
                return next == WeightedFlowLogRecordStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedFlowLogRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted flow log record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
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

    pub fn weighted_total(&self) -> f64 {
        ((self.sequence as f64) * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedFlowLogRecord {
        WeightedFlowLogRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            retries: self.retries,
            sequence: self.sequence,
            cost: self.cost,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a weighted flow log record in status idle.
pub fn new_weighted_flow_log_record(code: &str, title: &str) -> WeightedFlowLogRecord {
    WeightedFlowLogRecord {
        code: code.to_string(),
        title: title.to_string(),
        timezone: "".to_string(),
        retries: 0,
        sequence: 0,
        cost: 0.0,
        attributes: BTreeMap::new(),
        status: WeightedFlowLogRecordStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_weighted_flow_log_record("code-993", "title-101");
        assert_eq!(item.status, WeightedFlowLogRecordStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_weighted_flow_log_record("code-993", "title-101");
        if item.move_to(WeightedFlowLogRecordStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedFlowLogRecordStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_weighted_flow_log_record("code-993", "title-101");
        item.status = WeightedFlowLogRecordStatus::Stopped;
        assert!(
            item.move_to(WeightedFlowLogRecordStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
