use super::*;

use acme_kit::text;

/// Where a default peering record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultPeeringRecordStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DefaultPeeringRecordStatus {
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

impl std::fmt::Display for DefaultPeeringRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default peering record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPeeringRecord {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub cluster: String,
    pub tags: Vec<String>,
    pub ceiling: f64,
    pub sequence: i64,
    pub cipher: String,
    pub phone: String,
    pub status: DefaultPeeringRecordStatus,
}

impl DefaultPeeringRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultPeeringRecordStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultPeeringRecordStatus) -> bool {
        match self.status {
            DefaultPeeringRecordStatus::Idle => {
                match next {
                    DefaultPeeringRecordStatus::Ready | DefaultPeeringRecordStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultPeeringRecordStatus::Warming => {
                match next {
                    DefaultPeeringRecordStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultPeeringRecordStatus::Ready => {
                match next {
                    DefaultPeeringRecordStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultPeeringRecordStatus::Draining => {
                match next {
                    DefaultPeeringRecordStatus::Stopped => {
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

    pub fn move_to(&mut self, next: DefaultPeeringRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default peering record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default peering record", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_total(&self) -> f64 {
        ((self.sequence as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultPeeringRecord {
        DefaultPeeringRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            cluster: self.cluster.clone(),
            tags: self.tags.clone(),
            ceiling: self.ceiling,
            sequence: self.sequence,
            cipher: self.cipher.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a default peering record in status idle.
pub fn new_default_peering_record(code: &str, name: &str) -> DefaultPeeringRecord {
    DefaultPeeringRecord {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        cluster: "".to_string(),
        tags: Vec::new(),
        ceiling: 0.0,
        sequence: 0,
        cipher: "".to_string(),
        phone: "".to_string(),
        status: DefaultPeeringRecordStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_default_peering_record("code-25", "name-25");
        assert_eq!(item.status, DefaultPeeringRecordStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_default_peering_record("code-25", "name-25");
        if item.move_to(DefaultPeeringRecordStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultPeeringRecordStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_default_peering_record("code-25", "name-25");
        item.status = DefaultPeeringRecordStatus::Stopped;
        assert!(
            item.move_to(DefaultPeeringRecordStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
