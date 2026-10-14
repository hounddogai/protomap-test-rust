use super::*;

use acme_kit::text;

/// Where a legacy policy agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyPolicyAgreementStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LegacyPolicyAgreementStatus {
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

impl std::fmt::Display for LegacyPolicyAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy policy agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPolicyAgreement {
    pub code: String,
    pub title: String,
    pub category: String,
    pub external_ref: String,
    pub amount: f64,
    pub created_at: i64,
    pub tags: Vec<String>,
    pub channel: String,
    pub margin: f64,
    pub capacity: i64,
    pub status: LegacyPolicyAgreementStatus,
}

impl LegacyPolicyAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyPolicyAgreementStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyPolicyAgreementStatus) -> bool {
        match self.status {
            LegacyPolicyAgreementStatus::Draft => {
                return next == LegacyPolicyAgreementStatus::Closed;
            }
            LegacyPolicyAgreementStatus::Pending => {
                match next {
                    LegacyPolicyAgreementStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyPolicyAgreementStatus::Active => {
                match next {
                    LegacyPolicyAgreementStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyPolicyAgreementStatus::Suspended => {
                return next == LegacyPolicyAgreementStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyPolicyAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy policy agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.created_at as f64 - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyPolicyAgreement {
        LegacyPolicyAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            external_ref: self.external_ref.clone(),
            amount: self.amount,
            created_at: self.created_at,
            tags: self.tags.clone(),
            channel: self.channel.clone(),
            margin: self.margin,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a legacy policy agreement in status draft.
pub fn new_legacy_policy_agreement(code: &str, title: &str) -> LegacyPolicyAgreement {
    LegacyPolicyAgreement {
        code: code.to_string(),
        title: title.to_string(),
        category: "".to_string(),
        external_ref: "".to_string(),
        amount: 0.0,
        created_at: 0,
        tags: Vec::new(),
        channel: "".to_string(),
        margin: 0.0,
        capacity: 0,
        status: LegacyPolicyAgreementStatus::Draft,
    }
}
