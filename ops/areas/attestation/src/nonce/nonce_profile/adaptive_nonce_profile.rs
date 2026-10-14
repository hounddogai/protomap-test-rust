use super::*;

use acme_kit::text;

/// Where a adaptive nonce profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveNonceProfileStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AdaptiveNonceProfileStatus {
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

impl std::fmt::Display for AdaptiveNonceProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive nonce profile as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceProfile {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub locale: String,
    pub batch_size: i64,
    pub status: AdaptiveNonceProfileStatus,
}

impl AdaptiveNonceProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveNonceProfileStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveNonceProfileStatus) -> bool {
        match self.status {
            AdaptiveNonceProfileStatus::Planned => {
                match next {
                    AdaptiveNonceProfileStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveNonceProfileStatus::Scheduled => {
                match next {
                    AdaptiveNonceProfileStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveNonceProfileStatus::InProgress => {
                match next {
                    AdaptiveNonceProfileStatus::Completed | AdaptiveNonceProfileStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveNonceProfileStatus::Completed => {
                return next == AdaptiveNonceProfileStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveNonceProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive nonce profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.batch_size as f64
    }

    pub fn copy(&self) -> AdaptiveNonceProfile {
        AdaptiveNonceProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            locale: self.locale.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a adaptive nonce profile in status planned.
pub fn new_adaptive_nonce_profile(code: &str, title: &str) -> AdaptiveNonceProfile {
    AdaptiveNonceProfile {
        code: code.to_string(),
        title: title.to_string(),
        checksum: "".to_string(),
        locale: "".to_string(),
        batch_size: 0,
        status: AdaptiveNonceProfileStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_adaptive_nonce_profile("code-911", "title-570");
        assert_eq!(item.status, AdaptiveNonceProfileStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_adaptive_nonce_profile("code-911", "title-570");
        if item.move_to(AdaptiveNonceProfileStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveNonceProfileStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_adaptive_nonce_profile("code-911", "title-570");
        item.status = AdaptiveNonceProfileStatus::Cancelled;
        assert!(
            item.move_to(AdaptiveNonceProfileStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
