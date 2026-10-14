use super::*;

use acme_kit::text;

/// Where a sink slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkSlotStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SinkSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SinkSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a sink slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkSlot {
    pub code: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub expires_at: i64,
    pub maximum: f64,
    pub unit: String,
    pub capacity: i64,
    pub cipher: String,
    pub status: SinkSlotStatus,
}

impl SinkSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkSlotStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkSlotStatus) -> bool {
        match self.status {
            SinkSlotStatus::Created => {
                return next == SinkSlotStatus::Published || next == SinkSlotStatus::Retired;
            }
            SinkSlotStatus::Validated => {
                match next {
                    SinkSlotStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkSlotStatus::Published => {
                return next == SinkSlotStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkSlot {
        SinkSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            expires_at: self.expires_at,
            maximum: self.maximum,
            unit: self.unit.clone(),
            capacity: self.capacity,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a sink slot in status created.
pub fn new_sink_slot(code: &str, label: &str) -> SinkSlot {
    SinkSlot {
        code: code.to_string(),
        label: label.to_string(),
        ttl_seconds: 0,
        expires_at: 0,
        maximum: 0.0,
        unit: "".to_string(),
        capacity: 0,
        cipher: "".to_string(),
        status: SinkSlotStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_sink_slot("code-8", "label-170");
        assert_eq!(item.status, SinkSlotStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_sink_slot("code-8", "label-170");
        if item.move_to(SinkSlotStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkSlotStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_sink_slot("code-8", "label-170");
        item.status = SinkSlotStatus::Retired;
        assert!(item.move_to(SinkSlotStatus::Created).is_err(), "expected move to fail");
    }
}
