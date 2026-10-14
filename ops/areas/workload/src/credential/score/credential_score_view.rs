use super::*;

/// How screens and exports show a credential score.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialScoreView {
    pub code: String,
    pub label_text: String,
    pub discount: String,
    pub region: String,
    pub channel_text: String,
    pub country: String,
    pub summary: String,
    pub source: String,
    pub pinned_text: String,
    pub quantity: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_credential_score_view(item: &CredentialScore) -> CredentialScoreView {
    CredentialScoreView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        discount: format!("{:.2}", item.discount),
        region: item.region.clone(),
        channel_text: item.channel.clone(),
        country: item.country.clone(),
        summary: item.summary.clone(),
        source: item.source.clone(),
        pinned_text: format!("{}", item.pinned),
        quantity: item.quantity.to_string(),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_score_views(items: &[CredentialScore]) -> Vec<CredentialScoreView> {
    let mut views: Vec<CredentialScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_score_view(item));
    }
    views
}

pub fn sum_credential_score_view(view: &CredentialScoreView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_discount + (parsed_quantity as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_score("code-369", "label-937");
        let view = to_credential_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
