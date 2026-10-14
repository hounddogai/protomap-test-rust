use super::*;

use acme_kit::text;

/// Where a batched binding threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedBindingThresholdStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BatchedBindingThresholdStatus {
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

impl std::fmt::Display for BatchedBindingThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched binding threshold, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingThreshold {
    pub code: String,
    pub label: String,
    pub key_id: String,
    pub cipher: String,
    pub tenant_id: String,
    pub active: bool,
    pub updated_at: i64,
    pub threshold: f64,
    pub checksum: String,
    pub revision: i64,
    pub timezone: String,
    pub status: BatchedBindingThresholdStatus,
}

impl BatchedBindingThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedBindingThresholdStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedBindingThresholdStatus) -> bool {
        match self.status {
            BatchedBindingThresholdStatus::Idle => {
                return next == BatchedBindingThresholdStatus::Warming
                    || next == BatchedBindingThresholdStatus::Stopped;
            }
            BatchedBindingThresholdStatus::Warming => {
                return next == BatchedBindingThresholdStatus::Draining
                    || next == BatchedBindingThresholdStatus::Stopped;
            }
            BatchedBindingThresholdStatus::Ready => {
                return next == BatchedBindingThresholdStatus::Draining
                    || next == BatchedBindingThresholdStatus::Stopped;
            }
            BatchedBindingThresholdStatus::Draining => {
                return next == BatchedBindingThresholdStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedBindingThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched binding threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.updated_at as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedBindingThreshold {
        BatchedBindingThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            key_id: self.key_id.clone(),
            cipher: self.cipher.clone(),
            tenant_id: self.tenant_id.clone(),
            active: self.active,
            updated_at: self.updated_at,
            threshold: self.threshold,
            checksum: self.checksum.clone(),
            revision: self.revision,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched binding threshold in status idle.
pub fn new_batched_binding_threshold(code: &str, label: &str) -> BatchedBindingThreshold {
    BatchedBindingThreshold {
        code: code.to_string(),
        label: label.to_string(),
        key_id: "".to_string(),
        cipher: "".to_string(),
        tenant_id: "".to_string(),
        active: false,
        updated_at: 0,
        threshold: 0.0,
        checksum: "".to_string(),
        revision: 0,
        timezone: "".to_string(),
        status: BatchedBindingThresholdStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_batched_binding_threshold("code-107", "label-835");
        assert_eq!(item.status, BatchedBindingThresholdStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_batched_binding_threshold("code-107", "label-835");
        if item.move_to(BatchedBindingThresholdStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedBindingThresholdStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_batched_binding_threshold("code-107", "label-835");
        item.status = BatchedBindingThresholdStatus::Stopped;
        assert!(
            item.move_to(BatchedBindingThresholdStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
