use actix_web::{Result};
use reqwest::{Client as HttpClient};
use oauth2::{
    basic::{BasicClient, BasicErrorResponseType, BasicTokenType},
    Client, EmptyExtraTokenFields, RedirectUrl, RevocationErrorResponseType,
    RevocationUrl, Scope, StandardErrorResponse, StandardRevocableToken, StandardTokenIntrospectionResponse,
    StandardTokenResponse, TokenUrl, AuthorizationCode, AuthUrl, ClientId, ClientSecret, CsrfToken,
    reqwest::{async_http_client}, url::Url
};



pub mod google_auth {
    use oauth2::{RequestTokenError, TokenResponse};
    use serde::{Deserialize, Serialize};
    use super::*;


    #[derive(Deserialize, Serialize)]
    pub struct GoogleUserResult {
        pub id: String,
        pub picture: String,
        pub email: String
    }

    pub fn get_client() -> Client<
        StandardErrorResponse<BasicErrorResponseType>,
        StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
        BasicTokenType,
        StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
        StandardRevocableToken,
        StandardErrorResponse<RevocationErrorResponseType>
    > {
        let google_redirect_url = Url::parse("http://localhost:3000/auth/google/callback").expect("Invalid URL");
        BasicClient::new(
            ClientId::new("654695336089-b8r03jppmi2tj13mga5bicsctla4gjb3.apps.googleusercontent.com".to_string()),
            Some(ClientSecret::new("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".to_string())),
            AuthUrl::new("https://accounts.google.com/o/oauth2/auth".to_string()).expect("Invalid URL"),
            Some(TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string()).expect("Invalid URL")),
        )
        .set_redirect_uri(RedirectUrl::new(String::from(google_redirect_url)).expect("Invalid URL"))
        .set_revocation_uri(RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string()).expect("Invalid URL"))

    }

    pub fn get_auth_url(client: &BasicClient) -> String {
        let (authorize_url, _csrf_state) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/calendar".parse().unwrap(),
            ))
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/plus.me".to_string(),
            ))
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/userinfo.email".to_string(),
            ))
            .url();
        authorize_url.to_string()
    }

    pub async fn get_token(client: &BasicClient, code: AuthorizationCode) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>, String> {
        let token_result = client
            .exchange_code(code)
            .request_async(async_http_client).await;

        match token_result {
            Ok(token) => {
                Ok(token)
            }
            Err(err) => {
                match err {
                    RequestTokenError::ServerResponse(s) => Err(s.to_string()),
                    RequestTokenError::Request(r) => Err(r.to_string()),
                    RequestTokenError::Parse(e, _) => Err(e.to_string()),
                    RequestTokenError::Other(o) => Err(o.to_string())
                }
            }
        }
    }

    pub async fn get_user_info(response: StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>) -> Result<GoogleUserResult, String> {
        let client = HttpClient::new();
        let url = Url::parse("https://www.googleapis.com/oauth2/v1/userinfo?alt=json").unwrap();
        let response = client.get(url).bearer_auth(response.access_token().secret()).send().await;
        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<GoogleUserResult>().await {
                        Ok(user) => Ok(user),
                        Err(err) => Err(err.to_string()),
                    }
                } else {
                    Err("cannot access profile".to_string())
                }
            },
            Err(err) => Err(err.to_string()),
        }
    }
}