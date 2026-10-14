use super::*;

use acme_kit::text;

/// Where a sink allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkAllocationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SinkAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SinkAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a sink allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkAllocation {
    pub code: String,
    pub label: String,
    pub quantity: i64,
    pub cluster: String,
    pub tags: Vec<String>,
    pub checksum: String,
    pub summary: String,
    pub ttl_seconds: i64,
    pub port: i64,
    pub priority: i64,
    pub score: f64,
    pub status: SinkAllocationStatus,
}

impl SinkAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkAllocationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkAllocationStatus) -> bool {
        match self.status {
            SinkAllocationStatus::Requested => {
                match next {
                    SinkAllocationStatus::Granted | SinkAllocationStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkAllocationStatus::Granted => {
                return next == SinkAllocationStatus::Revoked;
            }
            SinkAllocationStatus::Revoked => {
                return next == SinkAllocationStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkAllocation {
        SinkAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            quantity: self.quantity,
            cluster: self.cluster.clone(),
            tags: self.tags.clone(),
            checksum: self.checksum.clone(),
            summary: self.summary.clone(),
            ttl_seconds: self.ttl_seconds,
            port: self.port,
            priority: self.priority,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a sink allocation in status requested.
pub fn new_sink_allocation(code: &str, label: &str) -> SinkAllocation {
    SinkAllocation {
        code: code.to_string(),
        label: label.to_string(),
        quantity: 0,
        cluster: "".to_string(),
        tags: Vec::new(),
        checksum: "".to_string(),
        summary: "".to_string(),
        ttl_seconds: 0,
        port: 0,
        priority: 0,
        score: 0.0,
        status: SinkAllocationStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_sink_allocation("code-882", "label-512");
        assert_eq!(item.status, SinkAllocationStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_sink_allocation("code-882", "label-512");
        if item.move_to(SinkAllocationStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkAllocationStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_sink_allocation("code-882", "label-512");
        item.status = SinkAllocationStatus::Expired;
        assert!(
            item.move_to(SinkAllocationStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
