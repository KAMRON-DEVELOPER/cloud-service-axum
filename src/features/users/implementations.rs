use crate::{
    features::users::models::{User, UserRole, UserStatus},
    utilities::errors::AppError,
};
use crate::{
    features::users::{
        models::{OAuthUser, Provider},
        schemas::{ContinueWithEmailSchema, GithubOAuthUser, GoogleOAuthUser},
    },
    services::database::Database,
};
use bcrypt::verify;
use validator::Validate;

impl From<GoogleOAuthUser> for OAuthUser {
    fn from(g: GoogleOAuthUser) -> Self {
        Self {
            id: g.sub,
            provider: Provider::Google,
            username: None,
            full_name: g.name,
            email: g.email,
            password: None,
            picture: g.picture,
            phone_number: g.phone_number,
            created_at: None,
        }
    }
}

impl From<GithubOAuthUser> for OAuthUser {
    fn from(g: GithubOAuthUser) -> Self {
        Self {
            id: g.id.to_string(),
            provider: Provider::Github,
            username: Some(g.login.clone()),
            full_name: None,
            email: g.email,
            password: None,
            picture: Some(g.avatar_url),
            phone_number: None,
            created_at: None,
        }
    }
}

impl ContinueWithEmailSchema {
    pub async fn verify(&self, database: &Database) -> Result<Option<User>, AppError> {
        self.validate()?;

        // let maybe_oauth_user = sqlx::query_as!(
        //     OAuthUser,
        //     r#"
        //         SELECT
        //             id,
        //             provider AS "provider: Provider",
        //             username,
        //             full_name,
        //             email,
        //             phone_number,
        //             password,
        //             picture,
        //             created_at
        //         FROM oauth_users WHERE id = $1
        //     "#,
        //     self.email
        // )
        // .fetch_optional(&database.pool)
        // .await?;

        let maybe_user = sqlx::query_as!(
            User,
            r#"
                SELECT
                    id,
                    full_name,
                    email,
                    phone_number,
                    password,
                    picture,
                    role AS "role: UserRole",
                    status AS "status: UserStatus",
                    email_verified,
                    oauth_user_id,
                    created_at,
                    updated_at
                FROM users WHERE email = $1
            "#,
            self.email
        )
        .fetch_optional(&database.pool)
        .await?;

        if let Some(user) = maybe_user {
            let verified = verify(&self.password, &user.password)?;

            if !verified {
                return Err(AppError::ValidationError(
                    "Password is incorrect".to_string(),
                ));
            }
            return Ok(Some(user));
        }

        Ok(None)
    }
}

// impl SignUpSchema {
//     pub async fn verify(self, database: &Database) -> Result<User, AppError> {
//         self.validate()?;

//         let maybe_user = sqlx::query_as!(
//             User,
//             r#"
//                 SELECT
//                     id,
//                     first_name,
//                     last_name,
//                     email,
//                     phone_number,
//                     password,
//                     picture,
//                     role AS "role: UserRole",
//                     status AS "status: UserStatus",
//                     email_verified,
//                     oauth_user_id,
//                     created_at,
//                     updated_at
//                 FROM users WHERE email = $1
//             "#,
//             self.email
//         )
//         .fetch_optional(&database.pool)
//         .await?;

//         let user = maybe_user
//             .ok_or_else(|| AppError::NotFoundError("User not found with this email".to_string()))?;

//         let verified = verify(&self.password, &user.password)?;

//         if !verified {
//             return Err(AppError::NotFoundError(
//                 "Password is not correct".to_string(),
//             ));
//         }

//         Ok(user)
//     }
// }
