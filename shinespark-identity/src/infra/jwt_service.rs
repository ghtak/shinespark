use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use shinespark::config::JwtConfig;

use crate::entities::UserAggregate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String, // user UUID
    pub exp: usize,
    pub roles: Option<Vec<i64>>,
    pub token_type: String, // "access" | "refresh"
}

#[derive(Debug, Clone, Serialize)]
pub struct JwtTokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub refresh_token_expires_at: DateTime<Utc>,
}

pub trait JwtService: Send + Sync + 'static {
    fn create(&self, aggregate: &UserAggregate) -> shinespark::Result<JwtTokenPair>;
    fn verify(&self, token: &str) -> shinespark::Result<JwtClaims>;
}

pub struct HS256JwtService {
    secret: String,
    access_token_minutes: i64,
    refresh_token_hours: i64,
}

impl HS256JwtService {
    pub fn new(config: &JwtConfig) -> Self {
        Self {
            secret: config.secret.clone(),
            access_token_minutes: config.access_token_minutes,
            refresh_token_hours: config.refresh_token_hours,
        }
    }
}

impl JwtService for HS256JwtService {
    fn create(&self, aggregate: &UserAggregate) -> shinespark::Result<JwtTokenPair> {
        let now = Utc::now();
        let access_exp = (now + Duration::minutes(self.access_token_minutes)).timestamp() as usize;
        let refresh_expires_at = now + Duration::hours(self.refresh_token_hours);
        let refresh_exp = refresh_expires_at.timestamp() as usize;

        let encoding_key = EncodingKey::from_secret(self.secret.as_bytes());
        let header = Header::new(Algorithm::HS256);

        let access_token = encode(
            &header,
            &JwtClaims {
                sub: aggregate.user.uid.to_string(),
                exp: access_exp,
                roles: Some(aggregate.role_ids.clone()),
                token_type: "access".to_string(),
            },
            &encoding_key,
        )
        .map_err(|e| {
            shinespark::Error::Internal(anyhow::anyhow!(e).context("failed to encode access token"))
        })?;

        let refresh_token = encode(
            &header,
            &JwtClaims {
                sub: aggregate.user.uid.to_string(),
                exp: refresh_exp,
                roles: None,
                token_type: "refresh".to_string(),
            },
            &encoding_key,
        )
        .map_err(|e| {
            shinespark::Error::Internal(
                anyhow::anyhow!(e).context("failed to encode refresh token"),
            )
        })?;

        Ok(JwtTokenPair {
            access_token,
            refresh_token,
            refresh_token_expires_at: refresh_expires_at,
        })
    }

    fn verify(&self, token: &str) -> shinespark::Result<JwtClaims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = 0;
        decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map(|data| data.claims)
        .map_err(|_| shinespark::Error::UnAuthorized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{User, UserAggregate, UserStatus};
    use uuid::Uuid;

    fn make_service() -> HS256JwtService {
        HS256JwtService {
            secret: "test-secret".to_string(),
            access_token_minutes: 30,
            refresh_token_hours: 24,
        }
    }

    fn make_aggregate() -> UserAggregate {
        UserAggregate {
            user: User {
                id: 1,
                uid: Uuid::new_v4(),
                name: "test".to_string(),
                email: "test@example.com".to_string(),
                status: UserStatus::Active,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            role_ids: vec![1, 2],
            identities: vec![],
        }
    }

    #[test]
    fn test_create_and_verify_access_token() {
        let svc = make_service();
        let agg = make_aggregate();
        let pair = svc.create(&agg).unwrap();
        let claims = svc.verify(&pair.access_token).unwrap();
        assert_eq!(claims.sub, agg.user.uid.to_string());
        assert_eq!(claims.token_type, "access");
        assert_eq!(claims.roles, Some(vec![1, 2]));
    }

    #[test]
    fn test_create_and_verify_refresh_token() {
        let svc = make_service();
        let agg = make_aggregate();
        let pair = svc.create(&agg).unwrap();
        let claims = svc.verify(&pair.refresh_token).unwrap();
        assert_eq!(claims.sub, agg.user.uid.to_string());
        assert_eq!(claims.token_type, "refresh");
        assert_eq!(claims.roles, None);
    }

    #[test]
    fn test_tampered_token_returns_error() {
        let svc = make_service();
        let result = svc.verify("tampered.token.value");
        assert!(result.is_err());
    }

    #[test]
    fn test_expired_token_returns_error() {
        let svc = HS256JwtService {
            secret: "test-secret".to_string(),
            access_token_minutes: -1, // already expired
            refresh_token_hours: 24,
        };
        let agg = make_aggregate();
        let pair = svc.create(&agg).unwrap();
        let result = svc.verify(&pair.access_token);
        assert!(result.is_err());
    }
}
