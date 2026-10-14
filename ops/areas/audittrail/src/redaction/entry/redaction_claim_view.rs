use super::*;

/// How screens and exports show a redaction claim.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionClaimView {
    pub code: String,
    pub name: String,
    pub margin_text: String,
    pub audience: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_redaction_claim_view(item: &RedactionClaim) -> RedactionClaimView {
    RedactionClaimView {
        code: item.code.clone(),
        name: item.name.clone(),
        margin_text: format!("{:.2}", item.margin),
        audience: item.audience.clone(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_redaction_claim_views(items: &[RedactionClaim]) -> Vec<RedactionClaimView> {
    let mut views: Vec<RedactionClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_redaction_claim_view(item));
    }
    views
}

pub fn sum_redaction_claim_view(view: &RedactionClaimView) -> Result<f64, String> {
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_margin + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_redaction_claim("code-702", "name-780");
        let view = to_redaction_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_redaction_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
