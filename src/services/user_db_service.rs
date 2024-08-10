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
                        id          INTEGER PRIMARY KEY,
                        email       TEXT NOT NULL UNIQUE,
                        password    TEXT NOT NULL,
                        displayName TEXT NOT NULL UNIQUE,
                        uuid        TEXT NOT NULL UNIQUE
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
            "INSERT INTO user (email, password, displayName, uuid) VALUES (?1, ?2, ?3, ?4)",
            (email, password, display_name, uuid),
        ) {
            Ok(_) => {
                return Ok(());
            }
            Err(err) => {
                println!("{:?}", err);
                return Err(UserDbError::GenericError);
            }
        }
    }
}
