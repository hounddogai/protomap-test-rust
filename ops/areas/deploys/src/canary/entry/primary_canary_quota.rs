use super::*;

use acme_kit::text;

/// Where a primary canary quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryCanaryQuotaStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PrimaryCanaryQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PrimaryCanaryQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary canary quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanaryQuota {
    pub code: String,
    pub label: String,
    pub minimum: f64,
    pub maximum: f64,
    pub updated_at: i64,
    pub status: PrimaryCanaryQuotaStatus,
}

impl PrimaryCanaryQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryCanaryQuotaStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryCanaryQuotaStatus) -> bool {
        match self.status {
            PrimaryCanaryQuotaStatus::Created => {
                return next == PrimaryCanaryQuotaStatus::Published || next == PrimaryCanaryQuotaStatus::Retired;
            }
            PrimaryCanaryQuotaStatus::Validated => {
                match next {
                    PrimaryCanaryQuotaStatus::Published | PrimaryCanaryQuotaStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryCanaryQuotaStatus::Published => {
                return next == PrimaryCanaryQuotaStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryCanaryQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary canary quota {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "primary canary quota", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.maximum - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryCanaryQuota {
        PrimaryCanaryQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            minimum: self.minimum,
            maximum: self.maximum,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary canary quota in status created.
pub fn new_primary_canary_quota(code: &str, label: &str) -> PrimaryCanaryQuota {
    PrimaryCanaryQuota {
        code: code.to_string(),
        label: label.to_string(),
        minimum: 0.0,
        maximum: 0.0,
        updated_at: 0,
        status: PrimaryCanaryQuotaStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_primary_canary_quota("code-70", "label-3");
        assert_eq!(item.status, PrimaryCanaryQuotaStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_primary_canary_quota("code-70", "label-3");
        if item.move_to(PrimaryCanaryQuotaStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryCanaryQuotaStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_primary_canary_quota("code-70", "label-3");
        item.status = PrimaryCanaryQuotaStatus::Retired;
        assert!(
            item.move_to(PrimaryCanaryQuotaStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
