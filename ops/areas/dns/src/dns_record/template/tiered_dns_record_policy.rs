use super::*;

use acme_kit::text;

/// Where a tiered dns record policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredDnsRecordPolicyStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TieredDnsRecordPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TieredDnsRecordPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered dns record policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordPolicy {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub phone: String,
    pub reviewer: String,
    pub title: String,
    pub rotation_days: i64,
    pub amount: f64,
    pub visible: bool,
    pub cluster: String,
    pub approved: bool,
    pub status: TieredDnsRecordPolicyStatus,
}

impl TieredDnsRecordPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredDnsRecordPolicyStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredDnsRecordPolicyStatus) -> bool {
        match self.status {
            TieredDnsRecordPolicyStatus::Created => {
                return next == TieredDnsRecordPolicyStatus::Published || next == TieredDnsRecordPolicyStatus::Retired;
            }
            TieredDnsRecordPolicyStatus::Validated => {
                return next == TieredDnsRecordPolicyStatus::Published;
            }
            TieredDnsRecordPolicyStatus::Published => {
                return next == TieredDnsRecordPolicyStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredDnsRecordPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered dns record policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tiered dns record policy", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.rotation_days as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredDnsRecordPolicy {
        TieredDnsRecordPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            phone: self.phone.clone(),
            reviewer: self.reviewer.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            amount: self.amount,
            visible: self.visible,
            cluster: self.cluster.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a tiered dns record policy in status created.
pub fn new_tiered_dns_record_policy(code: &str, label: &str) -> TieredDnsRecordPolicy {
    TieredDnsRecordPolicy {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        phone: "".to_string(),
        reviewer: "".to_string(),
        title: "".to_string(),
        rotation_days: 0,
        amount: 0.0,
        visible: false,
        cluster: "".to_string(),
        approved: false,
        status: TieredDnsRecordPolicyStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_tiered_dns_record_policy("code-317", "label-906");
        assert_eq!(item.status, TieredDnsRecordPolicyStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_tiered_dns_record_policy("code-317", "label-906");
        if item.move_to(TieredDnsRecordPolicyStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredDnsRecordPolicyStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_tiered_dns_record_policy("code-317", "label-906");
        item.status = TieredDnsRecordPolicyStatus::Retired;
        assert!(
            item.move_to(TieredDnsRecordPolicyStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
