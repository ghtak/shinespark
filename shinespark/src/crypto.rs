pub mod password {
    use crate::config::Argon2Config;
    use argon2::Argon2;
    use password_hash::{
        Ident, Output, ParamsString, PasswordHash, PasswordHasher,
        PasswordVerifier, SaltString,
    };

    pub trait PasswordService: Send + Sync + 'static {
        fn hash_password(&self, password: &[u8]) -> crate::Result<String>;
        fn verify_password(
            &self,
            password: &[u8],
            hash: &str,
        ) -> crate::Result<()>;
        fn needs_rehash(&self, hash: &str) -> bool;
    }

    pub struct Argon2PasswordService {
        params: argon2::Params,
    }

    impl Argon2PasswordService {
        pub fn new(config: &Argon2Config) -> crate::Result<Self> {
            let params = argon2::Params::new(
                config.memory_kib,
                config.iterations,
                config.parallelism,
                None,
            )
            .map_err(|e| {
                crate::Error::Unexpected(
                    anyhow::Error::new(e).context(
                        "failed to initialize argon2 parameters with the given configuration",
                    ),
                )
            })?;
            Ok(Self { params })
        }
    }

    impl PasswordService for Argon2PasswordService {
        fn hash_password(&self, password: &[u8]) -> crate::Result<String> {
            let salt =
                SaltString::generate(&mut password_hash::rand_core::OsRng);
            let argon2 = Argon2::new(
                argon2::Algorithm::Argon2id,
                argon2::Version::V0x13,
                self.params.clone(),
            );
            let hash = argon2.hash_password(password, &salt).map_err(|e| {
                crate::Error::Unexpected(
                    anyhow::Error::new(e)
                        .context("failed to hash password using argon2"),
                )
            })?;
            Ok(hash.to_string())
        }

        fn verify_password(
            &self,
            password: &[u8],
            hash: &str,
        ) -> crate::Result<()> {
            let hash =
                PasswordHash::new(hash).map_err(|e| {
                    crate::Error::Unexpected(anyhow::Error::new(e).context(
                        "invalid password hash format in argon2 service",
                    ))
                })?;
            match Argon2::default().verify_password(password, &hash) {
                Ok(_) => Ok(()),
                Err(argon2::password_hash::Error::Password) => {
                    Err(crate::Error::UnAuthorized)
                }
                Err(e) => Err(crate::Error::Unexpected(
                    anyhow::Error::new(e).context(
                        "failed to verify password hash in argon2 service",
                    ),
                )),
            }
        }

        fn needs_rehash(&self, hash: &str) -> bool {
            let hash = match PasswordHash::new(hash) {
                Ok(h) => h,
                Err(_) => return true,
            };

            // 1. Check Algorithm
            if hash.algorithm.as_str() != "argon2id" {
                return true;
            }

            // 2. Check Version
            if hash.version != Some(argon2::Version::V0x13.into()) {
                return true;
            }

            // 3. Check Params
            match argon2::Params::try_from(&hash) {
                Ok(params) => {
                    params.m_cost() != self.params.m_cost()
                        || params.t_cost() != self.params.t_cost()
                        || params.p_cost() != self.params.p_cost()
                }
                Err(_) => true,
            }
        }
    }

    pub struct Pbkdf2PasswordService;

    impl PasswordService for Pbkdf2PasswordService {
        fn hash_password(&self, password: &[u8]) -> crate::Result<String> {
            let salt =
                SaltString::generate(&mut password_hash::rand_core::OsRng);
            let hash =
                pbkdf2::Pbkdf2.hash_password(password, &salt).map_err(|e| {
                    crate::Error::Unexpected(
                        anyhow::Error::new(e)
                            .context("failed to hash password using pbkdf2"),
                    )
                })?;
            Ok(hash.to_string())
        }

        fn verify_password(
            &self,
            password: &[u8],
            hash: &str,
        ) -> crate::Result<()> {
            let hash =
                PasswordHash::new(hash).map_err(|e| {
                    crate::Error::Unexpected(anyhow::Error::new(e).context(
                        "invalid password hash format in pbkdf2 service",
                    ))
                })?;
            match pbkdf2::Pbkdf2.verify_password(password, &hash) {
                Ok(_) => Ok(()),
                Err(pbkdf2::password_hash::Error::Password) => {
                    Err(crate::Error::UnAuthorized)
                }
                Err(e) => Err(crate::Error::Unexpected(
                    anyhow::Error::new(e).context(
                        "failed to verify password hash in pbkdf2 service",
                    ),
                )),
            }
        }

        fn needs_rehash(&self, hash: &str) -> bool {
            let hash = match PasswordHash::new(hash) {
                Ok(h) => h,
                Err(_) => return true,
            };

            hash.algorithm.as_str() != "pbkdf2-sha256"
        }
    }

    pub struct NoopPasswordService;

    impl PasswordService for NoopPasswordService {
        fn hash_password(&self, password: &[u8]) -> crate::Result<String> {
            let mut password_vec = password.to_vec();
            if password_vec.len() < 10 {
                password_vec.resize(10, 0);
            }
            let salt =
                SaltString::generate(&mut password_hash::rand_core::OsRng);
            let output =
                Output::new(&password_vec).map_err(|e| {
                    crate::Error::Unexpected(anyhow::Error::new(e).context(
                        "failed to create output hash in noop service",
                    ))
                })?;

            let hash = PasswordHash {
                algorithm: Ident::new("noop").unwrap(),
                version: None,
                params: ParamsString::new(),
                salt: Some(salt.as_salt()),
                hash: Some(output),
            };
            Ok(hash.to_string())
        }

        fn verify_password(
            &self,
            password: &[u8],
            hash: &str,
        ) -> crate::Result<()> {
            let hash =
                PasswordHash::new(hash).map_err(|e| {
                    crate::Error::Unexpected(anyhow::Error::new(e).context(
                        "invalid password hash format in noop service",
                    ))
                })?;

            let mut password_vec = password.to_vec();
            if password_vec.len() < 10 {
                password_vec.resize(10, 0);
            }
            let output =
                Output::new(&password_vec).map_err(|e| {
                    crate::Error::Unexpected(anyhow::Error::new(e).context(
                        "failed to verify password hash in noop service",
                    ))
                })?;

            if hash.hash == Some(output) {
                Ok(())
            } else {
                Err(crate::Error::UnAuthorized)
            }
        }

        fn needs_rehash(&self, hash: &str) -> bool {
            let hash = match PasswordHash::new(hash) {
                Ok(h) => h,
                Err(_) => return true,
            };

            hash.algorithm.as_str() != "noop"
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_noop_service() {
            let password = "passwd".as_bytes();
            let service = NoopPasswordService;
            let hash = service.hash_password(password).unwrap();
            println!("{}", hash);
            service.verify_password(password, &hash).unwrap();
            assert!(!service.needs_rehash(&hash));
        }

        #[test]
        fn test_argon2_service() {
            let password = "passwd".as_bytes();
            let config = Argon2Config {
                memory_kib: 8,
                iterations: 1,
                parallelism: 1,
            };
            let service = Argon2PasswordService::new(&config).unwrap();
            let hash = service.hash_password(password).unwrap();
            println!("{}", hash);
            service.verify_password(password, &hash).unwrap();
            assert!(!service.needs_rehash(&hash));

            // Test rehash with different config
            let new_config = Argon2Config {
                memory_kib: 16,
                iterations: 1,
                parallelism: 1,
            };
            let new_service = Argon2PasswordService::new(&new_config).unwrap();
            assert!(new_service.needs_rehash(&hash));
        }

        #[test]
        fn test_pbkdf2_service() {
            let password = "passwd".as_bytes();
            let service = Pbkdf2PasswordService;
            let hash = service.hash_password(password).unwrap();
            println!("{}", hash);
            service.verify_password(password, &hash).unwrap();
            assert!(!service.needs_rehash(&hash));
        }

        #[test]
        fn test_cross_algorithm_rehash() {
            let password = "passwd".as_bytes();
            let pbkdf2_service = Pbkdf2PasswordService;
            let pbkdf2_hash = pbkdf2_service.hash_password(password).unwrap();

            let argon2_config = Argon2Config {
                memory_kib: 8,
                iterations: 1,
                parallelism: 1,
            };
            let argon2_service =
                Argon2PasswordService::new(&argon2_config).unwrap();

            // PBKDF2 hash should need rehash when checked by Argon2 service
            assert!(argon2_service.needs_rehash(&pbkdf2_hash));
        }
    }
}
