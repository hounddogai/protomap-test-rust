use super::*;

use acme_kit::text;

/// Where a binding adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingAdjustmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BindingAdjustmentStatus {
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

impl std::fmt::Display for BindingAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One binding adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingAdjustment {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub discount: f64,
    pub updated_at: i64,
    pub status: BindingAdjustmentStatus,
}

impl BindingAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingAdjustmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingAdjustmentStatus) -> bool {
        match self.status {
            BindingAdjustmentStatus::New => {
                return next == BindingAdjustmentStatus::Done;
            }
            BindingAdjustmentStatus::Queued => {
                match next {
                    BindingAdjustmentStatus::Done | BindingAdjustmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingAdjustmentStatus::Running => {
                match next {
                    BindingAdjustmentStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingAdjustmentStatus::Done => {
                match next {
                    BindingAdjustmentStatus::Failed => {
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

    pub fn move_to(&mut self, next: BindingAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding adjustment {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.updated_at as f64 + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingAdjustment {
        BindingAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            discount: self.discount,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a binding adjustment in status new.
pub fn new_binding_adjustment(code: &str, name: &str) -> BindingAdjustment {
    BindingAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        discount: 0.0,
        updated_at: 0,
        status: BindingAdjustmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_binding_adjustment("code-466", "name-853");
        assert_eq!(item.status, BindingAdjustmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_binding_adjustment("code-466", "name-853");
        if item.move_to(BindingAdjustmentStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingAdjustmentStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_binding_adjustment("code-466", "name-853");
        item.status = BindingAdjustmentStatus::Failed;
        assert!(
            item.move_to(BindingAdjustmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
