use rusqlite::Connection;

#[derive(Debug)]
pub enum UserDbError {
    GenericError,
    UserWithEmailAlreadyExist,
    UserWithDisplayNameAlreadyExist,
}

#[derive(Debug)]
pub struct UserDbService {
    pub conn: Connection,
}

impl UserDbService {
    pub fn connect() -> Result<Self, UserDbError> {
        match Connection::open("./db-collections/users.db") {
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
            "INSERT INTO user (email, password, displayName, uuid, emailVerified) VALUES (?1, ?2, ?3, ?4, ?5)",
            (email, password, display_name, uuid, 0),
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
                            if err_msg.contains("user.displayName)") {
                                UserDbError::UserWithDisplayNameAlreadyExist
                            } else if err_msg.contains("user.email)") {
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
                return Err(db_err);
            }
        }
    }
}
