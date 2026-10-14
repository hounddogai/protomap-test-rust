use super::*;

use acme_kit::text;

/// Where a binding segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingSegmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BindingSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for BindingSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a binding segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingSegment {
    pub code: String,
    pub label: String,
    pub key_id: String,
    pub quantity: i64,
    pub version: i64,
    pub score: f64,
    pub reviewer: String,
    pub channel: String,
    pub cluster: String,
    pub namespace: String,
    pub status: BindingSegmentStatus,
}

impl BindingSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingSegmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingSegmentStatus) -> bool {
        match self.status {
            BindingSegmentStatus::Planned => {
                return next == BindingSegmentStatus::Scheduled || next == BindingSegmentStatus::InProgress;
            }
            BindingSegmentStatus::Scheduled => {
                match next {
                    BindingSegmentStatus::Completed | BindingSegmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingSegmentStatus::InProgress => {
                return next == BindingSegmentStatus::Cancelled;
            }
            BindingSegmentStatus::Completed => {
                match next {
                    BindingSegmentStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: BindingSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "binding segment", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.score + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingSegment {
        BindingSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            key_id: self.key_id.clone(),
            quantity: self.quantity,
            version: self.version,
            score: self.score,
            reviewer: self.reviewer.clone(),
            channel: self.channel.clone(),
            cluster: self.cluster.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a binding segment in status planned.
pub fn new_binding_segment(code: &str, label: &str) -> BindingSegment {
    BindingSegment {
        code: code.to_string(),
        label: label.to_string(),
        key_id: "".to_string(),
        quantity: 0,
        version: 0,
        score: 0.0,
        reviewer: "".to_string(),
        channel: "".to_string(),
        cluster: "".to_string(),
        namespace: "".to_string(),
        status: BindingSegmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_binding_segment("code-632", "label-203");
        assert_eq!(item.status, BindingSegmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_binding_segment("code-632", "label-203");
        if item.move_to(BindingSegmentStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingSegmentStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_binding_segment("code-632", "label-203");
        item.status = BindingSegmentStatus::Cancelled;
        assert!(
            item.move_to(BindingSegmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
