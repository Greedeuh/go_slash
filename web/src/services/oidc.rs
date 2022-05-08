use openidconnect::{
    core::{CoreClient, CoreIdTokenClaims, CoreIdTokenVerifier, CoreResponseType},
    reqwest::async_http_client,
    AuthenticationFlow, AuthorizationCode, CsrfToken, Nonce, OAuth2TokenResponse, Scope,
};

#[cfg_attr(feature = "mock", faux::create)]
pub struct OidcService {
    client: CoreClient,
}

#[derive(Debug, PartialEq, Clone)]
pub enum OidcServiceError {
    Ko,
}
// #[cfg_attr(test, mockable)]

#[derive(Debug, PartialEq, Clone)]
pub struct TokenRes {
    pub mail: String,
}

#[cfg_attr(feature = "mock", faux::methods)]
impl OidcService {
    pub fn new(client: CoreClient) -> Self {
        Self { client }
    }

    pub fn redirect(&self) -> Result<(String, String), OidcServiceError> {
        let (auth_url, _csrf_token, nonce) = self
            .client
            .authorize_url(
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("email".to_string()))
            .url();

        Ok((auth_url.to_string(), nonce.secret().to_string()))
    }

    pub async fn retrieve_token(
        &self,
        code: &str,
        nonce: &str,
    ) -> Result<TokenRes, OidcServiceError> {
        let token_response = self
            .client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(async_http_client)
            .await
            .map_err(|e| {
                error!("Fail getting token from google: {:?}", e);
                OidcServiceError::Ko
            })?;

        println!(
            "Google returned access token:\n{}\n",
            token_response.access_token().secret()
        );
        println!("Google returned scopes: {:?}", token_response.scopes());

        let id_token_verifier: CoreIdTokenVerifier = self.client.id_token_verifier();
        let id_token_claims: &CoreIdTokenClaims = token_response
            .extra_fields()
            .id_token()
            .expect("Server did not return an ID token")
            .claims(&id_token_verifier, &Nonce::new(nonce.to_string()))
            .map_err(|e| {
                error!(
                    "Fail getting id_token from token_response: {:?} because {:?}",
                    token_response, e
                );
                OidcServiceError::Ko
            })?;

        let mail = match id_token_claims.email() {
            Some(mail) => mail.to_string(),
            None => {
                error!(
                    "Google user do not have mail and it's weird... token_response: {:?}",
                    token_response
                );
                return Err(OidcServiceError::Ko);
            }
        };

        Ok(TokenRes { mail })
    }
}
