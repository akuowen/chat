use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use sqlx::postgres::PgListener;
use sqlx::PgPool;

use crate::{AppError, User};

impl User {
    pub async fn find_by_email(
        email: &str,
        pool: &PgPool,
    ) -> anyhow::Result<Option<Self>, AppError> {
        let user =
            sqlx::query_as(r#"SELECT id,fullname,email,created_at FROM users WHERE email = $1"#)
                .bind(email)
                .fetch_optional(pool)
                .await?;
        Ok(user)
    }

    pub async fn create(
        fullname: &str,
        email: &str,
        password: &str,
        pool: &PgPool,
    ) -> anyhow::Result<Self, AppError> {
        let passwd_hash = hash_password(password)?;
        let user = sqlx::query_as(
            r#"INSERT INTO users (fullname,email,password_hash) VALUES ($1,$2,$3) RETURNING id,fullname,email,created_at"#,
        )
        .bind(fullname)
        .bind(email)
        .bind(passwd_hash)
        .fetch_one(pool)
        .await?;
        Ok(user)
    }

    ///verify email and password
    pub async fn verify(
        email: &str,
        password: &str,
        pool: &PgPool,
    ) -> anyhow::Result<Option<Self>, AppError> {
        let mut listener = PgListener::connect_with(pool).await?;
        listener.listen("channel_name").await?;

        let user: Option<User> =
            sqlx::query_as(r#"SELECT id,fullname,email,created_at FROM users WHERE email = $1"#)
                .bind(email)
                .fetch_optional(pool)
                .await?;

        match user {
            Some(user) => {
                // let password_hash = mem::take(&mut user.password_hash);
                let is_valid = verify_password(password, &user.password_hash.clone())?;
                if is_valid {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();
    let password_hash = PasswordHash::new(password_hash)?;

    // Verify password
    let is_valid = argon2
        .verify_password(password.as_bytes(), &password_hash)
        .is_ok();

    Ok(is_valid)
}

#[cfg(test)]
mod tests {

    use super::*;
    use sqlx_db_tester::TestPg;
    use std::path::Path;

    #[tokio::test]
    async fn test_user_create() -> anyhow::Result<()> {
        let tdb = TestPg::new(
            "postgres://postgres:akuowen@localhost:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let user = User::create("akuowen", "akuowen2023@gmail,com", "passwd", &pool).await?;
        assert_eq!(&user.email, "akuowen2023@gmail,com");
        assert_eq!(&user.fullname, "akuowen");
        assert_eq!(&user.password_hash, "");

        let result = User::find_by_email("akuowen2023@gmail,com", &pool).await?;
        assert!(result.is_some());
        Ok(())
    }
}
