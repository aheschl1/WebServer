use crate::server_core::ftp::status::ConnectionState;
/**
 * Hash the password using sha256.
 */
pub fn hash_password(password: &str) -> String{
    // hash the password using a secure hashing algorithm.
    // return the hashed password.
    sha256::digest(password)
}

pub fn auth_can_access_file(file: &str, auth_state: ConnectionState) -> bool{
    // check if the file can be accessed by the user.
    if auth_state == ConnectionState::Disconnected || auth_state == ConnectionState::NotLoggedIn{
        return false;
    }
    // if the file is inside of "public" or "shared" directories, it is not privelaged.
    if file.contains("public") || file.contains("shared"){
        return true;
    }
    // if authenticated, give full access.
    return auth_state == ConnectionState::LoggedIn;
}