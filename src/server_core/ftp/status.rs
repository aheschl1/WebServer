pub enum ConnectionState{
    NotLoggedIn,
    Disconnected,
    LoggedIn,
    Annonymous
}

pub enum TransferMode{
    Active,
    Passive,
    Stream,
    Block,
    Compressed
}

// convert from string
impl From<&str> for TransferMode{
    fn from(s: &str) -> Self{
        match s{
            "A" => TransferMode::Active,
            "P" => TransferMode::Passive,
            "S" => TransferMode::Stream,
            "B" => TransferMode::Block,
            "C" => TransferMode::Compressed,
            _ => TransferMode::Stream
        }
    }
}
// to string
impl std::fmt::Display for TransferMode{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result{
        match self{
            TransferMode::Active => write!(f, "Active"),
            TransferMode::Passive => write!(f, "Passive"),
            TransferMode::Stream => write!(f, "Stream"),
            TransferMode::Block => write!(f, "Block"),
            TransferMode::Compressed => write!(f, "Compressed")
        }
    }
}

pub enum TransferType{
    Ascii,  // 7-bit ASCII data for text files
    Binary, // 8-bit bytes for images
    EBCDIC, // Extended Binary Coded Decimal Interchange Code
}

impl PartialEq for TransferType{
    fn eq(&self, other: &Self) -> bool{
        match (self, other){
            (TransferType::Ascii, TransferType::Ascii) => true,
            (TransferType::Binary, TransferType::Binary) => true,
            (TransferType::EBCDIC, TransferType::EBCDIC) => true,
            _ => false
        }
    }
}

// conversion from strings
impl From<&str> for TransferType{
    fn from(s: &str) -> Self{
        match s{
            "A" => TransferType::Ascii,
            "I" => TransferType::Binary,
            "E" => TransferType::EBCDIC,
            _ => TransferType::Ascii
        }
    }
}

// to str
impl std::fmt::Display for TransferType{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result{
        match self{
            TransferType::Ascii => write!(f, "ASCII"),
            TransferType::Binary => write!(f, "Binary"),
            TransferType::EBCDIC => write!(f, "EBCDIC")
        }
    }
}

impl PartialEq for ConnectionState{
    fn eq(&self, other: &Self) -> bool{
        match (self, other){
            (ConnectionState::NotLoggedIn, ConnectionState::NotLoggedIn) => true,
            (ConnectionState::Disconnected, ConnectionState::Disconnected) => true,
            (ConnectionState::LoggedIn, ConnectionState::LoggedIn) => true,
            (ConnectionState::Annonymous, ConnectionState::Annonymous) => true,
            _ => false
        }
    }
}

pub enum TransferStructure{
    File,
    Record,
    Page
}
// equality
impl PartialEq for TransferStructure{
    fn eq(&self, other: &Self) -> bool{
        match (self, other){
            (TransferStructure::File, TransferStructure::File) => true,
            (TransferStructure::Record, TransferStructure::Record) => true,
            (TransferStructure::Page, TransferStructure::Page) => true,
            _ => false
        }
    }
}

// conversion from strings
impl From<&str> for TransferStructure{
    fn from(s: &str) -> Self{
        match s{
            "F" => TransferStructure::File,
            "R" => TransferStructure::Record,
            "P" => TransferStructure::Page,
            _ => TransferStructure::File
        }
    }
}

// to str
impl std::fmt::Display for TransferStructure{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result{
        match self{
            TransferStructure::File => write!(f, "File"),
            TransferStructure::Record => write!(f, "Record"),
            TransferStructure::Page => write!(f, "Page")
        }
    }
}

// Make them all cloneable

impl Clone for TransferMode{
    fn clone(&self) -> Self{
        match self{
            TransferMode::Active => TransferMode::Active,
            TransferMode::Passive => TransferMode::Passive,
            TransferMode::Stream => TransferMode::Stream,
            TransferMode::Block => TransferMode::Block,
            TransferMode::Compressed => TransferMode::Compressed
        }
    }
}

impl Clone for TransferType{
    fn clone(&self) -> Self{
        match self{
            TransferType::Ascii => TransferType::Ascii,
            TransferType::Binary => TransferType::Binary,
            TransferType::EBCDIC => TransferType::EBCDIC
        }
    }
}

impl Clone for ConnectionState{
    fn clone(&self) -> Self{
        match self{
            ConnectionState::NotLoggedIn => ConnectionState::NotLoggedIn,
            ConnectionState::Disconnected => ConnectionState::Disconnected,
            ConnectionState::LoggedIn => ConnectionState::LoggedIn,
            ConnectionState::Annonymous => ConnectionState::Annonymous
        }
    }
}

impl Clone for TransferStructure{
    fn clone(&self) -> Self{
        match self{
            TransferStructure::File => TransferStructure::File,
            TransferStructure::Record => TransferStructure::Record,
            TransferStructure::Page => TransferStructure::Page
        }
    }
}
