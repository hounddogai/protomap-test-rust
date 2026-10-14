use super::*;

use acme_kit::text;

/// Where a nonce adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceAdjustmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl NonceAdjustmentStatus {
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

impl std::fmt::Display for NonceAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A nonce adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceAdjustment {
    pub code: String,
    pub name: String,
    pub description: String,
    pub visible: bool,
    pub region: String,
    pub ceiling: f64,
    pub sequence: i64,
    pub expires_at: i64,
    pub currency: String,
    pub notes: String,
    pub status: NonceAdjustmentStatus,
}

impl NonceAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceAdjustmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceAdjustmentStatus) -> bool {
        match self.status {
            NonceAdjustmentStatus::New => {
                match next {
                    NonceAdjustmentStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceAdjustmentStatus::Queued => {
                return next == NonceAdjustmentStatus::Running || next == NonceAdjustmentStatus::Done;
            }
            NonceAdjustmentStatus::Running => {
                return next == NonceAdjustmentStatus::Done || next == NonceAdjustmentStatus::Failed;
            }
            NonceAdjustmentStatus::Done => {
                match next {
                    NonceAdjustmentStatus::Failed => {
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

    pub fn move_to(&mut self, next: NonceAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce adjustment {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.sequence as f64 - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceAdjustment {
        NonceAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            visible: self.visible,
            region: self.region.clone(),
            ceiling: self.ceiling,
            sequence: self.sequence,
            expires_at: self.expires_at,
            currency: self.currency.clone(),
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a nonce adjustment in status new.
pub fn new_nonce_adjustment(code: &str, name: &str) -> NonceAdjustment {
    NonceAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        visible: false,
        region: "".to_string(),
        ceiling: 0.0,
        sequence: 0,
        expires_at: 0,
        currency: "".to_string(),
        notes: "".to_string(),
        status: NonceAdjustmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_nonce_adjustment("code-410", "name-692");
        assert_eq!(item.status, NonceAdjustmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_nonce_adjustment("code-410", "name-692");
        if item.move_to(NonceAdjustmentStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceAdjustmentStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_nonce_adjustment("code-410", "name-692");
        item.status = NonceAdjustmentStatus::Failed;
        assert!(
            item.move_to(NonceAdjustmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
