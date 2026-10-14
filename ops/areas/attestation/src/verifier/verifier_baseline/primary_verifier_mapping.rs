use super::*;

use acme_kit::text;

/// Where a primary verifier mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryVerifierMappingStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PrimaryVerifierMappingStatus {
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

impl std::fmt::Display for PrimaryVerifierMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary verifier mapping. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierMapping {
    pub code: String,
    pub title: String,
    pub quota_units: i64,
    pub audience: String,
    pub weight: f64,
    pub discount: f64,
    pub summary: String,
    pub phone: String,
    pub label: String,
    pub port: i64,
    pub status: PrimaryVerifierMappingStatus,
}

impl PrimaryVerifierMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryVerifierMappingStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryVerifierMappingStatus) -> bool {
        match self.status {
            PrimaryVerifierMappingStatus::Draft => {
                match next {
                    PrimaryVerifierMappingStatus::Pending | PrimaryVerifierMappingStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryVerifierMappingStatus::Pending => {
                match next {
                    PrimaryVerifierMappingStatus::Active | PrimaryVerifierMappingStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryVerifierMappingStatus::Active => {
                match next {
                    PrimaryVerifierMappingStatus::Suspended | PrimaryVerifierMappingStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryVerifierMappingStatus::Suspended => {
                match next {
                    PrimaryVerifierMappingStatus::Closed => {
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

    pub fn move_to(&mut self, next: PrimaryVerifierMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary verifier mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.weight - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryVerifierMapping {
        PrimaryVerifierMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            quota_units: self.quota_units,
            audience: self.audience.clone(),
            weight: self.weight,
            discount: self.discount,
            summary: self.summary.clone(),
            phone: self.phone.clone(),
            label: self.label.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary verifier mapping in status draft.
pub fn new_primary_verifier_mapping(code: &str, title: &str) -> PrimaryVerifierMapping {
    PrimaryVerifierMapping {
        code: code.to_string(),
        title: title.to_string(),
        quota_units: 0,
        audience: "".to_string(),
        weight: 0.0,
        discount: 0.0,
        summary: "".to_string(),
        phone: "".to_string(),
        label: "".to_string(),
        port: 0,
        status: PrimaryVerifierMappingStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_primary_verifier_mapping("code-256", "title-383");
        assert_eq!(item.status, PrimaryVerifierMappingStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_primary_verifier_mapping("code-256", "title-383");
        if item.move_to(PrimaryVerifierMappingStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryVerifierMappingStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_primary_verifier_mapping("code-256", "title-383");
        item.status = PrimaryVerifierMappingStatus::Closed;
        assert!(
            item.move_to(PrimaryVerifierMappingStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
