use std::{fs, io};

use rusqlite::Connection;

#[derive(Debug)]
pub enum UserDbError {
    GenericError,
    UserWithEmailAlreadyExist,
    UserWithDisplayNameAlreadyExist,
    UserNotFound,
}

#[derive(Debug)]
pub struct UserDbService {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct User {
    pub uuid: String,
    pub display_name: String,
    pub email: String,
}

fn create_directory_if_not_exists(path: &str) -> io::Result<()> {
    if !std::path::Path::new(path).exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

impl UserDbService {
    pub fn connect(db_collected_root_dir: &str) -> Result<Self, UserDbError> {
        create_directory_if_not_exists(&db_collected_root_dir)
            .map_err(|_| UserDbError::GenericError)?;

        match Connection::open(format!("{}/users.db", db_collected_root_dir)) {
            Ok(conn) => {
                match conn.execute(
                    "CREATE TABLE IF NOT EXISTS user (
                        id               INTEGER PRIMARY KEY,
                        email            TEXT NOT NULL UNIQUE,
                        password         TEXT NOT NULL,
                        displayName      TEXT NOT NULL UNIQUE,
                        uuid             TEXT NOT NULL UNIQUE,
                        emailVerified    INTEGER NOT NULL
                    )",
                    (), // empty list of parameters.
                ) {
                    Ok(_) => {
                        return Ok(Self { conn });
                    }
                    Err(_) => {
                        return Err(UserDbError::GenericError);
                    }
                };
            }
            Err(_) => {
                return Err(UserDbError::GenericError);
            }
        };
    }

    pub fn add_user(
        &self,
        email: &str,
        password: &str,
        display_name: &str,
        uuid: &str,
    ) -> Result<(), UserDbError> {
        match self.conn.execute(
            "INSERT INTO user (email, password, displayName, uuid, emailVerified) VALUES (?1, ?2, ?3, ?4, 0)",
            (email, password, display_name, uuid),
        ) {
            Ok(_) => {
                return Ok(());
            }
            Err(err) => {
                log::error!("{:?}", err);
                let db_err = match err {
                    rusqlite::Error::SqliteFailure(sqlite_err, msg) => {
                        if sqlite_err.code == rusqlite::ErrorCode::ConstraintViolation {
                            let err_msg = msg.unwrap_or("-".to_string());
                            if err_msg.contains("user.displayName") {
                                UserDbError::UserWithDisplayNameAlreadyExist
                            } else if err_msg.contains("user.emailVerified") {
                                UserDbError::GenericError
                            } else if err_msg.contains("user.email") {
                                UserDbError::UserWithEmailAlreadyExist
                            } else {
                                UserDbError::GenericError
                            }
                        } else {
                            UserDbError::GenericError
                        }
                    }
                    _ => UserDbError::GenericError,
                };
                // log::error!("{:?}", db_err);
                return Err(db_err);
            }
        }
    }

    pub fn get_password_from_email(&self, email: &str) -> Result<String, UserDbError> {
        if let Ok(mut statement) = self
            .conn
            .prepare("SELECT password FROM user WHERE email=:email limit 1;")
        {
            if let Ok(user_iter) = statement.query_map(&[(":email", email)], |row| {
                let password: String = row.get(0)?;
                Ok(password)
            }) {
                let password_vec: Vec<_> = user_iter.collect();

                if let Some(selected_password) = password_vec.get(0) {
                    if let Ok(password) = selected_password {
                        return Ok(password.clone());
                    }
                }
            }
        }

        return Err(UserDbError::UserNotFound);
    }

    pub fn get_user_from_email(&self, email: &str) -> Result<User, UserDbError> {
        if let Ok(mut statement) = self
            .conn
            .prepare("SELECT uuid, displayName FROM user WHERE email=:email limit 1;")
        {
            if let Ok(user_iter) = statement.query_map(&[(":email", email)], |row| {
                Ok(User {
                    uuid: row.get(0)?,
                    display_name: row.get(1)?,
                    email: email.to_owned(),
                })
            }) {
                let user_vec: Vec<_> = user_iter.collect();

                if let Some(selected_user) = user_vec.get(0) {
                    if let Ok(user) = selected_user {
                        return Ok(user.clone());
                    }
                }
            }
        }

        return Err(UserDbError::UserNotFound);
    }
}
