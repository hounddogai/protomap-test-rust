use super::*;

use acme_kit::text;

/// Where a fallback binding adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackBindingAdjustmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FallbackBindingAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FallbackBindingAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback binding adjustment as the binding team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingAdjustment {
    pub code: String,
    pub name: String,
    pub score: f64,
    pub checksum: String,
    pub burst: i64,
    pub capacity: i64,
    pub status: FallbackBindingAdjustmentStatus,
}

impl FallbackBindingAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackBindingAdjustmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackBindingAdjustmentStatus) -> bool {
        match self.status {
            FallbackBindingAdjustmentStatus::Requested => {
                match next {
                    FallbackBindingAdjustmentStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackBindingAdjustmentStatus::Granted => {
                match next {
                    FallbackBindingAdjustmentStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackBindingAdjustmentStatus::Revoked => {
                return next == FallbackBindingAdjustmentStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackBindingAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback binding adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.burst as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackBindingAdjustment {
        FallbackBindingAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            score: self.score,
            checksum: self.checksum.clone(),
            burst: self.burst,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a fallback binding adjustment in status requested.
pub fn new_fallback_binding_adjustment(code: &str, name: &str) -> FallbackBindingAdjustment {
    FallbackBindingAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        score: 0.0,
        checksum: "".to_string(),
        burst: 0,
        capacity: 0,
        status: FallbackBindingAdjustmentStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_fallback_binding_adjustment("code-825", "name-51");
        assert_eq!(item.status, FallbackBindingAdjustmentStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_fallback_binding_adjustment("code-825", "name-51");
        if item.move_to(FallbackBindingAdjustmentStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackBindingAdjustmentStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_fallback_binding_adjustment("code-825", "name-51");
        item.status = FallbackBindingAdjustmentStatus::Expired;
        assert!(
            item.move_to(FallbackBindingAdjustmentStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
