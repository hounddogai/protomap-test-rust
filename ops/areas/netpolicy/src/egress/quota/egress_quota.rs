use super::*;

use acme_kit::text;

/// Where a egress quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressQuotaStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl EgressQuotaStatus {
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

impl std::fmt::Display for EgressQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A egress quota as the egress team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressQuota {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub summary: String,
    pub ends_at: i64,
    pub floor_value: f64,
    pub audience: String,
    pub cluster: String,
    pub locale: String,
    pub status: EgressQuotaStatus,
}

impl EgressQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressQuotaStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressQuotaStatus) -> bool {
        match self.status {
            EgressQuotaStatus::Draft => {
                match next {
                    EgressQuotaStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressQuotaStatus::Pending => {
                match next {
                    EgressQuotaStatus::Active | EgressQuotaStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressQuotaStatus::Active => {
                return next == EgressQuotaStatus::Closed;
            }
            EgressQuotaStatus::Suspended => {
                match next {
                    EgressQuotaStatus::Closed => {
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

    pub fn move_to(&mut self, next: EgressQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.ends_at as f64 + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressQuota {
        EgressQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            summary: self.summary.clone(),
            ends_at: self.ends_at,
            floor_value: self.floor_value,
            audience: self.audience.clone(),
            cluster: self.cluster.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a egress quota in status draft.
pub fn new_egress_quota(code: &str, label: &str) -> EgressQuota {
    EgressQuota {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        summary: "".to_string(),
        ends_at: 0,
        floor_value: 0.0,
        audience: "".to_string(),
        cluster: "".to_string(),
        locale: "".to_string(),
        status: EgressQuotaStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_egress_quota("code-455", "label-959");
        assert_eq!(item.status, EgressQuotaStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_egress_quota("code-455", "label-959");
        if item.move_to(EgressQuotaStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressQuotaStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_egress_quota("code-455", "label-959");
        item.status = EgressQuotaStatus::Closed;
        assert!(item.move_to(EgressQuotaStatus::Draft).is_err(), "expected move to fail");
    }
}
