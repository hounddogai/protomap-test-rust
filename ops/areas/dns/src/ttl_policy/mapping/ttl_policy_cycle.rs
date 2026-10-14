use super::*;

use acme_kit::text;

/// Where a ttl policy cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyCycleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TtlPolicyCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TtlPolicyCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One ttl policy cycle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyCycle {
    pub code: String,
    pub name: String,
    pub audience: String,
    pub fingerprint: String,
    pub title: String,
    pub contact_email: String,
    pub source: String,
    pub unit: String,
    pub label: String,
    pub active: bool,
    pub score: f64,
    pub status: TtlPolicyCycleStatus,
}

impl TtlPolicyCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyCycleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyCycleStatus) -> bool {
        match self.status {
            TtlPolicyCycleStatus::Created => {
                match next {
                    TtlPolicyCycleStatus::Published | TtlPolicyCycleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyCycleStatus::Validated => {
                match next {
                    TtlPolicyCycleStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyCycleStatus::Published => {
                match next {
                    TtlPolicyCycleStatus::Retired => {
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

    pub fn move_to(&mut self, next: TtlPolicyCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "ttl policy cycle", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> TtlPolicyCycle {
        TtlPolicyCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            audience: self.audience.clone(),
            fingerprint: self.fingerprint.clone(),
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            source: self.source.clone(),
            unit: self.unit.clone(),
            label: self.label.clone(),
            active: self.active,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a ttl policy cycle in status created.
pub fn new_ttl_policy_cycle(code: &str, name: &str) -> TtlPolicyCycle {
    TtlPolicyCycle {
        code: code.to_string(),
        name: name.to_string(),
        audience: "".to_string(),
        fingerprint: "".to_string(),
        title: "".to_string(),
        contact_email: "".to_string(),
        source: "".to_string(),
        unit: "".to_string(),
        label: "".to_string(),
        active: false,
        score: 0.0,
        status: TtlPolicyCycleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_ttl_policy_cycle("code-708", "name-962");
        assert_eq!(item.status, TtlPolicyCycleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_ttl_policy_cycle("code-708", "name-962");
        if item.move_to(TtlPolicyCycleStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyCycleStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_ttl_policy_cycle("code-708", "name-962");
        item.status = TtlPolicyCycleStatus::Retired;
        assert!(
            item.move_to(TtlPolicyCycleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
