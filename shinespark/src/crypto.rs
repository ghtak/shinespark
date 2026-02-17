pub mod password {

    use password_hash::{
        Ident, Output, ParamsString, PasswordHash, PasswordHasher,
    };

    pub struct NoopHasher {}

    #[derive(Clone, Debug, Default)]
    pub struct NoopParams {}

    impl TryFrom<&PasswordHash<'_>> for NoopParams {
        type Error = password_hash::Error;

        fn try_from(_value: &PasswordHash<'_>) -> Result<Self, Self::Error> {
            Ok(NoopParams {})
        }
    }

    impl TryInto<ParamsString> for NoopParams {
        type Error = password_hash::Error;

        fn try_into(self) -> Result<ParamsString, Self::Error> {
            Ok(ParamsString::new())
        }
    }

    impl PasswordHasher for NoopHasher {
        type Params = NoopParams;

        fn hash_password_customized<'a>(
            &self,
            password: &[u8],
            algorithm: Option<password_hash::Ident<'a>>,
            version: Option<password_hash::Decimal>,
            params: Self::Params,
            salt: impl Into<password_hash::Salt<'a>>,
        ) -> password_hash::Result<PasswordHash<'a>> {
            let mut password = password.to_vec();
            if password.len() < 10 {
                password.resize(10, 0);
            }

            let output = Output::new(password.as_slice())?;
            let algorithm = algorithm.unwrap_or(Ident::new("noop").unwrap());

            Ok(PasswordHash {
                algorithm: algorithm,
                version,
                params: params.try_into()?,
                salt: Some(salt.into()),
                hash: Some(output),
            })
        }
    }

    #[cfg(test)]
    mod test {

        use password_hash::{PasswordVerifier, SaltString, rand_core::OsRng};

        use super::*;

        #[test]
        fn test_noop_hasher() {
            let password = "passwd".as_bytes();
            let salt = SaltString::generate(&mut OsRng);
            let hasher = NoopHasher {};
            let hash = hasher.hash_password(password, &salt).unwrap();
            println!("{}", hash.to_string());
            hasher.verify_password(password, &hash).unwrap();
        }

        #[test]
        fn test_argon2_hasher() {
            let password = "passwd".as_bytes();
            let salt = SaltString::generate(&mut OsRng);
            let hasher = argon2::Argon2::default();
            let hash = hasher.hash_password(password, &salt).unwrap();
            println!("{}", hash.to_string());

            hasher.verify_password(password, &hash).unwrap();
        }

        #[test]
        fn test_sha2_hasher() {
            let password = "passwd".as_bytes();
            let salt = SaltString::generate(&mut OsRng);
            let password_hash =
                pbkdf2::Pbkdf2.hash_password(password, &salt).unwrap();
            println!("{}", password_hash.to_string());
            pbkdf2::Pbkdf2.verify_password(password, &password_hash).unwrap();
        }
    }
}
