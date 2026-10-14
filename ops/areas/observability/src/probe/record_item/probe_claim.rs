use super::*;

use acme_kit::text;

/// Where a probe claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeClaimStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ProbeClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ProbeClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A probe claim. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeClaim {
    pub code: String,
    pub name: String,
    pub category: String,
    pub port: i64,
    pub audience: String,
    pub status: ProbeClaimStatus,
}

impl ProbeClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeClaimStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeClaimStatus) -> bool {
        match self.status {
            ProbeClaimStatus::Created => {
                match next {
                    ProbeClaimStatus::Published | ProbeClaimStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeClaimStatus::Validated => {
                return next == ProbeClaimStatus::Published;
            }
            ProbeClaimStatus::Published => {
                return next == ProbeClaimStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ProbeClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.port as f64
    }

    pub fn copy(&self) -> ProbeClaim {
        ProbeClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            port: self.port,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a probe claim in status created.
pub fn new_probe_claim(code: &str, name: &str) -> ProbeClaim {
    ProbeClaim {
        code: code.to_string(),
        name: name.to_string(),
        category: "".to_string(),
        port: 0,
        audience: "".to_string(),
        status: ProbeClaimStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_probe_claim("code-410", "name-738");
        assert_eq!(item.status, ProbeClaimStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_probe_claim("code-410", "name-738");
        if item.move_to(ProbeClaimStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeClaimStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_probe_claim("code-410", "name-738");
        item.status = ProbeClaimStatus::Retired;
        assert!(
            item.move_to(ProbeClaimStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
