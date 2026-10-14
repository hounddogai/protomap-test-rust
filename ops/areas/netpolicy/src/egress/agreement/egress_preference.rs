use super::*;

use acme_kit::text;

/// Where a egress preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressPreferenceStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl EgressPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for EgressPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A egress preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressPreference {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub cluster: String,
    pub source: String,
    pub currency: String,
    pub score: f64,
    pub status: EgressPreferenceStatus,
}

impl EgressPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressPreferenceStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressPreferenceStatus) -> bool {
        match self.status {
            EgressPreferenceStatus::Created => {
                match next {
                    EgressPreferenceStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressPreferenceStatus::Validated => {
                match next {
                    EgressPreferenceStatus::Published | EgressPreferenceStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressPreferenceStatus::Published => {
                return next == EgressPreferenceStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EgressPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "egress preference", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> EgressPreference {
        EgressPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            cluster: self.cluster.clone(),
            source: self.source.clone(),
            currency: self.currency.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a egress preference in status created.
pub fn new_egress_preference(code: &str, title: &str) -> EgressPreference {
    EgressPreference {
        code: code.to_string(),
        title: title.to_string(),
        checksum: "".to_string(),
        cluster: "".to_string(),
        source: "".to_string(),
        currency: "".to_string(),
        score: 0.0,
        status: EgressPreferenceStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_egress_preference("code-839", "title-637");
        assert_eq!(item.status, EgressPreferenceStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_egress_preference("code-839", "title-637");
        if item.move_to(EgressPreferenceStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressPreferenceStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_egress_preference("code-839", "title-637");
        item.status = EgressPreferenceStatus::Retired;
        assert!(
            item.move_to(EgressPreferenceStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
