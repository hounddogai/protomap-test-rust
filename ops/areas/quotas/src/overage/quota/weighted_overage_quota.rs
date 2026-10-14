use super::*;

use acme_kit::text;

/// Where a weighted overage quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedOverageQuotaStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl WeightedOverageQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for WeightedOverageQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted overage quota as the overage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedOverageQuota {
    pub code: String,
    pub label: String,
    pub minimum: f64,
    pub host: String,
    pub namespace: String,
    pub status: WeightedOverageQuotaStatus,
}

impl WeightedOverageQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedOverageQuotaStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedOverageQuotaStatus) -> bool {
        match self.status {
            WeightedOverageQuotaStatus::Created => {
                match next {
                    WeightedOverageQuotaStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedOverageQuotaStatus::Validated => {
                match next {
                    WeightedOverageQuotaStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedOverageQuotaStatus::Published => {
                match next {
                    WeightedOverageQuotaStatus::Retired => {
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

    pub fn move_to(&mut self, next: WeightedOverageQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted overage quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "weighted overage quota", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.minimum
    }

    pub fn copy(&self) -> WeightedOverageQuota {
        WeightedOverageQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            minimum: self.minimum,
            host: self.host.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a weighted overage quota in status created.
pub fn new_weighted_overage_quota(code: &str, label: &str) -> WeightedOverageQuota {
    WeightedOverageQuota {
        code: code.to_string(),
        label: label.to_string(),
        minimum: 0.0,
        host: "".to_string(),
        namespace: "".to_string(),
        status: WeightedOverageQuotaStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_weighted_overage_quota("code-614", "label-502");
        assert_eq!(item.status, WeightedOverageQuotaStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_weighted_overage_quota("code-614", "label-502");
        if item.move_to(WeightedOverageQuotaStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedOverageQuotaStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_weighted_overage_quota("code-614", "label-502");
        item.status = WeightedOverageQuotaStatus::Retired;
        assert!(
            item.move_to(WeightedOverageQuotaStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
