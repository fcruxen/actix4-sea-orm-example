use actix_web::dev::ServiceRequest;
use actix_web::Error;

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, TokenData};
use serde::{Deserialize, Serialize};
use actix_web_httpauth::{extractors::bearer::{BearerAuth, Config}};
use actix_web_httpauth::extractors::AuthenticationError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub(crate) id: String,
    pub(crate) sub: String,
    pub(crate) app: String,
    pub(crate) exp: usize,
}


impl Claims {
    pub fn encode(&self) -> jsonwebtoken::errors::Result<String> {
        encode(&Header::default(), self, &EncodingKey::from_secret("secret".as_ref()))
    }

    pub fn decode(token: String) -> jsonwebtoken::errors::Result<TokenData<Claims>> {
        println!("Received token: {}", token.to_owned());
        decode::<Claims>(&token, &DecodingKey::from_secret("secret".as_ref()), &Validation::default())
    }
}

pub async fn bearer_auth_validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let config = req
        .app_data::<Config>()
        .map(|data| data.clone())
        .unwrap_or_else(Default::default);
    match Claims::decode(credentials.token().to_owned()) {
        Ok(_) => {
            Ok(req)
        }
        Err(err) => {
            println!("Error Decoding: {}", err);
            Err((AuthenticationError::from(config).into(), req))
        }
    }
}