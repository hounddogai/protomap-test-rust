use super::*;

use acme_kit::text;

/// Where a migration agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationAgreementStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl MigrationAgreementStatus {
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

impl std::fmt::Display for MigrationAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A migration agreement as the migration team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationAgreement {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub key_id: String,
    pub cluster: String,
    pub amount: f64,
    pub pinned: bool,
    pub active: bool,
    pub status: MigrationAgreementStatus,
}

impl MigrationAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationAgreementStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationAgreementStatus) -> bool {
        match self.status {
            MigrationAgreementStatus::Idle => {
                match next {
                    MigrationAgreementStatus::Warming | MigrationAgreementStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationAgreementStatus::Warming => {
                return next == MigrationAgreementStatus::Ready || next == MigrationAgreementStatus::Stopped;
            }
            MigrationAgreementStatus::Ready => {
                match next {
                    MigrationAgreementStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationAgreementStatus::Draining => {
                match next {
                    MigrationAgreementStatus::Stopped => {
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

    pub fn move_to(&mut self, next: MigrationAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "migration agreement", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> MigrationAgreement {
        MigrationAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            key_id: self.key_id.clone(),
            cluster: self.cluster.clone(),
            amount: self.amount,
            pinned: self.pinned,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a migration agreement in status idle.
pub fn new_migration_agreement(code: &str, label: &str) -> MigrationAgreement {
    MigrationAgreement {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        key_id: "".to_string(),
        cluster: "".to_string(),
        amount: 0.0,
        pinned: false,
        active: false,
        status: MigrationAgreementStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_migration_agreement("code-720", "label-21");
        assert_eq!(item.status, MigrationAgreementStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_migration_agreement("code-720", "label-21");
        if item.move_to(MigrationAgreementStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationAgreementStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_migration_agreement("code-720", "label-21");
        item.status = MigrationAgreementStatus::Stopped;
        assert!(
            item.move_to(MigrationAgreementStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
