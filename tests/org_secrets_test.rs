// Tests for calls to the /repos/{owner}/actions/secrets API.
mod mock_error;

use chrono::DateTime;
use mock_error::setup_error_handler;
use octocrab::{
    models::{
        orgs::secrets::{
            CreateOrganizationSecret, CreateOrganizationSecretResponse, OrganizationSecret,
            OrganizationSecrets, Visibility,
        },
        PublicKey,
    },
    Octocrab,
};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

const ORG: &str = "org";

async fn setup_get_api(template: ResponseTemplate, secrets_path: &str) -> MockServer {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/orgs/{ORG}/actions/secrets{secrets_path}")))
        .respond_with(template)
        .mount(&mock_server)
        .await;
    setup_error_handler(
        &mock_server,
        &format!("GET on /orgs/{ORG}/actions/secrets{secrets_path} was not received"),
    )
    .await;
    mock_server
}

async fn setup_put_api(template: ResponseTemplate, secrets_path: &str) -> MockServer {
    let mock_server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path(format!("/orgs/{ORG}/actions/secrets{secrets_path}")))
        .respond_with(template)
        .mount(&mock_server)
        .await;
    setup_error_handler(
        &mock_server,
        &format!("GET on /orgs/{ORG}/actions/secrets{secrets_path} was not received"),
    )
    .await;
    mock_server
}

fn setup_octocrab(uri: &str) -> Octocrab {
    Octocrab::builder().base_uri(uri).unwrap().build().unwrap()
}

#[tokio::test]
async fn should_return_org_secrets() {
    let org_secrets: OrganizationSecrets =
        serde_json::from_str(include_str!("resources/org_secrets.json")).unwrap();

    let template = ResponseTemplate::new(200).set_body_json(&org_secrets);
    let mock_server = setup_get_api(template, "").await;
    let client = setup_octocrab(&mock_server.uri());
    let org = client.orgs(ORG.to_owned());
    let secrets = org.secrets();
    let result = secrets.get_secrets().await;
    assert!(
        result.is_ok(),
        "expected successful result, got error: {:#?}",
        result
    );
    let item = result.unwrap();

    assert_eq!(item.total_count, 3);
    assert_eq!(item.secrets,vec![
            OrganizationSecret {
                name: String::from("GIST_ID"),
                created_at: DateTime::parse_from_rfc3339("2019-08-10T14:59:22Z").unwrap().into(),
                updated_at: DateTime::parse_from_rfc3339("2020-01-10T14:59:22Z").unwrap().into(),
                visibility: Visibility::Private,
                selected_repositories_url: None,
            },
            OrganizationSecret {
                name: String::from("DEPLOY_TOKEN"),
                created_at: DateTime::parse_from_rfc3339("2019-08-10T14:59:22Z").unwrap().into(),
                updated_at: DateTime::parse_from_rfc3339("2020-01-10T14:59:22Z").unwrap().into(),
                visibility: Visibility::All,
                selected_repositories_url: None,
            },
            OrganizationSecret {
                name: String::from("GH_TOKEN"),
                created_at: DateTime::parse_from_rfc3339("2019-08-10T14:59:22Z").unwrap().into(),
                updated_at: DateTime::parse_from_rfc3339("2020-01-10T14:59:22Z").unwrap().into(),
                visibility: Visibility::Selected,
                selected_repositories_url: Some(String::from("https://api.github.com/orgs/octo-org/actions/secrets/SUPER_SECRET/repositories")),
            },
        ]
    );
}

#[tokio::test]
async fn should_return_org_public_key() {
    let org_secrets: PublicKey =
        serde_json::from_str(include_str!("resources/org_public_key.json")).unwrap();

    let template = ResponseTemplate::new(200).set_body_json(&org_secrets);
    let mock_server = setup_get_api(template, "/public-key").await;
    let client = setup_octocrab(&mock_server.uri());
    let org = client.orgs(ORG.to_owned());
    let secrets = org.secrets();
    let result = secrets.get_public_key().await;
    assert!(
        result.is_ok(),
        "expected successful result, got error: {:#?}",
        result
    );
    let item = result.unwrap();

    assert_eq!(item.key_id, String::from("012345678912345678"));
    assert_eq!(
        item.key,
        String::from("2Sg8iYjAxxmI2LvUXpJjkYrMxURPc8r+dB7TJyvv1234")
    );
}

#[tokio::test]
async fn should_return_org_secret() {
    let org_secrets: OrganizationSecret =
        serde_json::from_str(include_str!("resources/org_secret.json")).unwrap();

    let template = ResponseTemplate::new(200).set_body_json(&org_secrets);
    let mock_server = setup_get_api(template, "/GH_TOKEN").await;
    let client = setup_octocrab(&mock_server.uri());
    let org = client.orgs(ORG.to_owned());
    let secrets = org.secrets();
    let result = secrets.get_secret("GH_TOKEN").await;
    assert!(
        result.is_ok(),
        "expected successful result, got error: {:#?}",
        result
    );
    let item = result.unwrap();
    assert_eq!(
        item,
        OrganizationSecret {
            name: String::from("GH_TOKEN"),
            created_at: DateTime::parse_from_rfc3339("2019-08-10T14:59:22Z")
                .unwrap()
                .into(),
            updated_at: DateTime::parse_from_rfc3339("2020-01-10T14:59:22Z")
                .unwrap()
                .into(),
            visibility: Visibility::Selected,
            selected_repositories_url: Some(String::from(
                "https://api.github.com/orgs/octo-org/actions/secrets/SUPER_SECRET/repositories"
            )),
        }
    );
}

#[tokio::test]
async fn should_add_secret() {
    let template = ResponseTemplate::new(201);
    let mock_server = setup_put_api(template, "/GH_TOKEN").await;
    let client = setup_octocrab(&mock_server.uri());
    let org = client.orgs(ORG.to_owned());
    let secrets = org.secrets();
    let result = secrets
        .create_or_update_secret(
            "GH_TOKEN",
            &CreateOrganizationSecret {
                key_id: "123456",
                encrypted_value: "some-b64-string",
                visibility: Visibility::Selected,
                selected_repository_ids: None,
            },
        )
        .await;
    assert!(
        result.is_ok(),
        "expected successful result, got error: {:#?}",
        result
    );
    let item = result.unwrap();
    assert_eq!(item, CreateOrganizationSecretResponse::Created);
}
