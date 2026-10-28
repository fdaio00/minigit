use std::{
    env,
    error::Error,
    fmt::{self, format, write},
    fs, io,
    path::{Path, PathBuf},
    process::ExitCode,
};

// use hex_litral::hex;
use sha2::{Digest, Sha256};

// -------------------------
// 1. Commands
// -------------------------

enum CommandType {
    Init,
    Add,
    Commit,
    Status,
    Log,
    HashObject(PathBuf),
}

struct Config {
    command_type: CommandType,
}

// -------------------------
// 2. Application Errors
// -------------------------

#[derive(Debug)]
enum AppError {
    MissingCommand,
    UnknownCommand(String),
    TooManyArguments,
    NotImplemented(&'static str),
    Io(io::Error),
    TooFewArgumnets,
    PathNotFound(&'static str),
    HashError,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::MissingCommand => {
                write!(f, "Missing command. Usage: minigit <command>")
            }
            AppError::UnknownCommand(command) => {
                write!(f, "Unknown command: {command}")
            }
            AppError::TooManyArguments => {
                write!(f, "Too many arguments")
            }
            AppError::HashError => {
                write!(f, "the file colud not be hashed!")
            }
            AppError::NotImplemented(command) => {
                write!(f, "Command '{command}' is not implemented yet")
            }
            AppError::Io(error) => {
                write!(f, "Filesystem error: {error}")
            }

            AppError::TooFewArgumnets => {
                write!(f, "Command has too few arguments")
            }
            AppError::PathNotFound(file_path) => {
                write!(f, "the file : {} is not found", { file_path })
            }
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        AppError::Io(error)
    }
}

// -------------------------
// 3. Parse CLI Arguments
// -------------------------

impl Config {
    fn build(args: &[String]) -> Result<Self, AppError> {
        let command = args.get(1).ok_or(AppError::MissingCommand)?;

        if args.len() > 2 {
            return Err(AppError::TooManyArguments);
        }

        let command_type = match command.as_str() {
            "init" => CommandType::Init,
            "add" => CommandType::Add,
            "commit" => CommandType::Commit,
            "status" => CommandType::Status,
            "hash-object" => {
                if args.len() < 3 {
                    return Err(AppError::TooFewArgumnets);
                }

                if args.len() > 3 {
                    return Err(AppError::TooManyArguments);
                }

                let path = args.get(2).ok_or(AppError::TooFewArgumnets)?;

                CommandType::HashObject(PathBuf::from(path.as_str()))
            }
            "log" => CommandType::Log,
            _ => {
                return Err(AppError::UnknownCommand(command.clone()));
            }
        };

        Ok(Self { command_type })
    }
}

// -------------------------
// 4. Repository Operations
// -------------------------
fn hash_file(data: &[u8]) -> String {
    let hash = Sha256::digest(data);
    hex::encode(hash)
}
fn initialize_repository(root: &Path) -> io::Result<PathBuf> {
    let git_dir = root.join(".minigit");

    // Fail if .minigit already exists.
    fs::create_dir(&git_dir)?;

    let setup_result = (|| -> io::Result<()> {
        fs::create_dir(git_dir.join("objects"))?;
        fs::create_dir_all(git_dir.join("refs").join("heads"))?;

        fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n")?;

        Ok(())
    })();

    if let Err(error) = setup_result {
        // Best-effort cleanup of partial initialization.
        if let Err(cleanup_error) = fs::remove_dir_all(&git_dir) {
            eprintln!(
                "Warning: could not clean up {}: {cleanup_error}",
                git_dir.display()
            );
        }

        return Err(error);
    }

    Ok(git_dir)
}

// -------------------------
// 5. Execute Commands
// -------------------------

fn execute_command(command_type: CommandType) -> Result<String, AppError> {
    match command_type {
        CommandType::Init => {
            let current_dir = env::current_dir()?;

            let repository = initialize_repository(&current_dir)?;

            Ok(format!(
                "Initialized MiniGit repository in {}",
                repository.display()
            ))
        }

        CommandType::Add => Err(AppError::NotImplemented("add")),
        CommandType::Commit => Err(AppError::NotImplemented("commit")),
        CommandType::Status => Err(AppError::NotImplemented("status")),
        CommandType::Log => Err(AppError::NotImplemented("log")),
        CommandType::HashObject(file_path) => {
            let data = fs::read(&file_path)?;
            let hashed = hash_file(&data);
            Ok(format!(
                "the file in {} has been hashed, the hash is : {}",
                &file_path.display(),
                hashed
            ))
        }
    }
}

// -------------------------
// 6. Application Runner
// -------------------------

fn run() -> Result<String, AppError> {
    let args: Vec<String> = env::args().collect();

    let config = Config::build(&args)?;

    execute_command(config.command_type)
}

// -------------------------
// 7. Program Entry Point
// -------------------------

fn main() -> ExitCode {
    match run() {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }

        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}
