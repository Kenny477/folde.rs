use clap::{Parser, Subcommand};
use serde_yaml::{Mapping, Sequence, Value};
use std::fs::{read_dir, DirEntry, File};
use std::path::PathBuf;

#[derive(Subcommand)]
enum Commands {
    #[clap(about = "Create directory tree from YAML file")]
    Push {
        #[clap(num_args = 1, required = true)]
        path: std::path::PathBuf,
        #[clap(short, long, default_value = "output.yaml")]
        src: std::path::PathBuf,
    },

    #[clap(about = "Create YAML file from directory tree")]
    Pull {
        #[clap(num_args = 1.., required = true)]
        path: Vec<std::path::PathBuf>,
        #[clap(short, long, default_value = "output.yaml")]
        dest: std::path::PathBuf,
    },
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn pull(paths: &Vec<PathBuf>, dest: &PathBuf) -> Result<Value, std::io::Error> {
    println!("Pulling from {:?}", paths);

    fn get_filename(path: &PathBuf) -> Result<String, std::io::Error> {
        let filename = path
            .file_name()
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid pathname",
            ))?
            .to_str()
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid pathname",
            ))?;
        Ok(filename.to_string())
    }

    fn build_yaml(path: &PathBuf) -> Result<Value, std::io::Error> {
        let filename = get_filename(path)?;
        // Handle file
        if path.is_file() {
            return Ok(Value::String(filename.to_string()));
        }
        // Handle directory
        else if path.is_dir() {
            // Get list of files in directory
            let dir = read_dir(path)?;
            // TODO: Handle error without panic
            let filelist: Vec<DirEntry> = dir.map(|e| e.unwrap()).collect();
            // If directory is empty, return directory name
            if filelist.len() == 0 {
                return Ok(Value::String(format!("{}/", filename.to_string())));
            }
            // Otherwise return a mapping of the directory name to a sequence of the files
            let mut seq = Sequence::new();
            for file in filelist {
                let path = file.path();
                let filename = get_filename(&path)?;
                let val = if path.is_dir() {
                    build_yaml(&path)?
                } else {
                    Value::String(filename.to_string())
                };
                // Build sequence
                seq.push(val);
            }
            let mut map = Mapping::new();
            map.insert(
                Value::String(format!("{}/", filename.to_string())),
                Value::Sequence(seq),
            );
            return Ok(Value::Mapping(map));
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Not a file or directory",
            ));
        }
    }

    let yaml = Value::Sequence(paths.iter().map(|path| build_yaml(path).unwrap()).collect());
    println!("{:?}", yaml);

    // Output YAML to file
    let outfile = File::create(dest).unwrap();
    serde_yaml::to_writer(outfile, &yaml).unwrap();
    return Ok(yaml);
}

fn push(path: &PathBuf, src: &PathBuf) -> Result<Value, std::io::Error> {
    println!("Pushing to {:?}", path);

    let file = std::fs::File::open(src)?;
    let yaml: Value = match serde_yaml::from_reader(&file) {
        Ok(yaml) => yaml,
        Err(e) => {
            println!("Error: {:?}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid YAML file",
            ));
        }
    };

    fn create_item(s: &str, root: &PathBuf) {
        if s.ends_with("/") {
            println!("Creating directory {:?}", s);
            std::fs::create_dir(root.join(s)).unwrap();
        } else {
            println!("Creating file {:?}", s);
            File::create(root.join(s)).unwrap();
        }
    }

    // Iterate over YAML
    fn generate_structure(val: &Value, root: &PathBuf) {
        match val {
            Value::String(s) => {
                create_item(s, root);
            }
            Value::Mapping(map) => {
                for (key, val) in map {
                    create_item(key.as_str().unwrap(), root);
                    generate_structure(val, &root.join(key.as_str().unwrap()));
                }
            }
            Value::Sequence(seq) => {
                for val in seq {
                    generate_structure(val, root);
                }
            }
            _ => {}
        }
    }

    println!("{:?}", yaml);
    generate_structure(&yaml, path);

    return Ok(yaml);
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Push { path, src } => push(path, src),
        Commands::Pull { path, dest } => pull(path, dest),
    };
}
