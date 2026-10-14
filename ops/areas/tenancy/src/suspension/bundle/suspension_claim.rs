use super::*;

use acme_kit::text;

/// Where a suspension claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionClaimStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SuspensionClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for SuspensionClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A suspension claim as the suspension team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionClaim {
    pub code: String,
    pub name: String,
    pub weight: f64,
    pub attempts: i64,
    pub maximum: f64,
    pub status: SuspensionClaimStatus,
}

impl SuspensionClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionClaimStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionClaimStatus) -> bool {
        match self.status {
            SuspensionClaimStatus::Draft => {
                match next {
                    SuspensionClaimStatus::Pending | SuspensionClaimStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionClaimStatus::Pending => {
                return next == SuspensionClaimStatus::Active || next == SuspensionClaimStatus::Closed;
            }
            SuspensionClaimStatus::Active => {
                return next == SuspensionClaimStatus::Suspended || next == SuspensionClaimStatus::Closed;
            }
            SuspensionClaimStatus::Suspended => {
                return next == SuspensionClaimStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SuspensionClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "suspension claim", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.weight + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionClaim {
        SuspensionClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            weight: self.weight,
            attempts: self.attempts,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a suspension claim in status draft.
pub fn new_suspension_claim(code: &str, name: &str) -> SuspensionClaim {
    SuspensionClaim {
        code: code.to_string(),
        name: name.to_string(),
        weight: 0.0,
        attempts: 0,
        maximum: 0.0,
        status: SuspensionClaimStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_suspension_claim("code-229", "name-160");
        assert_eq!(item.status, SuspensionClaimStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_suspension_claim("code-229", "name-160");
        if item.move_to(SuspensionClaimStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionClaimStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_suspension_claim("code-229", "name-160");
        item.status = SuspensionClaimStatus::Closed;
        assert!(
            item.move_to(SuspensionClaimStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
