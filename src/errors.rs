#[derive(Debug,)]
pub enum DBType{
    Postgres,
    Mongodb,
}
#[derive(Debug)]
pub enum DBError {
   QueryFailed(DBType),
   ConnectionFailed(DBType),
}

#[derive(Debug)]
pub enum AHError {
   InvalidCredential,
   UserInactive,
   AccountSuspended, 
}

#[derive(Debug)]
pub enum ErrorKind {
    DatabaseError(DBError),
    AuthError(AHError), 
}

impl std::fmt::Display for DBError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DBError::QueryFailed(db_type) => {
                match db_type {
                    DBType::Postgres => write!(f, "Postgres query failed"),
                    DBType::Mongodb => write!(f, "Mongodb query failed"),
                }
            },
            DBError::ConnectionFailed(db_type) => {
                match db_type {
                    DBType::Postgres => write!(f, "Postgres connection failed"),
                    DBType::Mongodb => write!(f, "Mongodb connection failed"),
                }
            },
        }
    }
}
impl std::error::Error for DBError{}

impl std::fmt::Display for AHError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AHError::InvalidCredential => write!(f, "Invalid credential"),
            AHError::UserInactive => write!(f, "User account is inactive"),
            AHError::AccountSuspended => write!(f, "User account is suspended"),
        }
    }
}
impl std::error::Error for AHError {}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::DatabaseError(db_error) => write!(f, "Database error: {}", db_error),
            ErrorKind::AuthError(auth_error) => write!(f, "Authentication error: {}", auth_error),
        }
    }
}
impl std::error::Error for ErrorKind {}