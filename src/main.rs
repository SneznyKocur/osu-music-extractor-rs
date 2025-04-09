use audiotags::Tag;
use std::fs;
use std::fs::DirEntry;
use std::io::{Write, stdin, stdout};
use std::path::{Path, PathBuf};
use rayon::prelude::*;
use std::env;

struct OsuFile {
    artist: String,
    title: String,
    sound_path: PathBuf,
}

fn escape_string(string: &str) -> String {
    let mut escaped_string = String::new();
    for c in string.escape_default() {
        escaped_string.push(c);
    }
    escaped_string
}

fn parse_osu(file: &DirEntry) -> OsuFile {
    let file_content: String = fs::read_to_string(file.path()).unwrap();
    let mut osufile: OsuFile = OsuFile {
        artist: String::new(),
        title: String::new(),
        sound_path: PathBuf::new(),
    };
    let file_path = file.path().canonicalize().unwrap();
    let directory: &Path = &file_path.parent().unwrap();
    for line in file_content.split('\n') {
        if !(line.contains(':')) {
            continue;
        }
        let split_line: (&str, &str) = line.split_once(':').unwrap();
        match split_line.0 {
            "AudioFilename" => {
                osufile.sound_path = directory.join(split_line.1.trim());
            }
            "Title" => {
                osufile.title = split_line.1.to_string().trim().to_string();
            }
            "Artist" => {
                osufile.artist = split_line.1.to_string().trim().to_string();
            }
            _ => (),
        }
    }
    osufile
}

fn copy(file_path: &DirEntry, output_dir: &Path) -> () {
    if !file_path.file_type().unwrap().is_file() {
        return;
    }
    let file_name: std::ffi::OsString = file_path.file_name();
    let binding = PathBuf::from(file_name);
    let extension: &str = match binding.extension() {
        Some(extension) => extension.to_str().unwrap(),
        None => {
            eprintln!("File {:#?} does not have an extension!", file_path);
            return;
        }
    };
    if !(extension == "osu") {
        return;
    }
    let osu_file: OsuFile = parse_osu(file_path);
    let sound_extension: &str = match osu_file.sound_path.extension() {
        Some(extension) => extension.to_str().unwrap(),
        None => {
            eprintln!(
                "Sound file {:#?} does not have an extension!",
                osu_file.sound_path
            );
            return;
        }
    };
    let output_file_name = &output_dir.join(
        escape_string(&osu_file.title.clone())
            + " - "
            + &escape_string(&osu_file.artist.clone())
            + "."
            + sound_extension,
    );
    let _ = fs::copy(osu_file.sound_path.as_path(), output_file_name);
    add_tag(output_file_name, osu_file);
}

fn add_tag(file_path: &PathBuf, osu_file: OsuFile) {
    let mut tag = match Tag::new().read_from_path(file_path) {
        Ok(tag) => tag,
        Err(_) => Box::new(audiotags::Id3v2Tag::default()),
    };
    tag.set_title(&osu_file.title);
    tag.set_artist(&osu_file.artist);
    let _ = tag.write_to_path(file_path.to_str().unwrap());
}

fn visit_dirs(dir: &DirEntry, output_dir: &Path) -> std::io::Result<()> {
    if dir.path().is_dir() {
        for entry in fs::read_dir(dir.path())? {
            let entry = entry?;
            let path = entry;
            if path.path().is_dir() {
                visit_dirs(&path, output_dir)?;
            } else {
                copy(&path, output_dir);
            }
        }
    }
    Ok(())
}
fn main() -> std::io::Result<()> {
    let mut songs_directory: String = String::new();
    let mut output_directory: String = String::new();
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        songs_directory = args[1].clone();
    }
    if args.len() > 2 {
        output_directory = args[2].clone();
    }
    if args.len() == 1 {
        print!("Enter osu songs directory:");
        let _ = stdout().flush();
        stdin()
            .read_line(&mut songs_directory)
            .expect("Failed to get user input!");
        print!("Enter output songs directory (will be created if it does not exist):");
        let _ = stdout().flush();
        stdin()
            .read_line(&mut output_directory)
            .expect("Failed to get user input!");
    }
    

    songs_directory = songs_directory.trim().to_string();
    output_directory = output_directory.trim().to_string();

    let songs_path: &Path = Path::new(songs_directory.as_str());
    if !songs_path.is_dir() {
        eprintln!("Songs directory does not exist!");
    }

    if output_directory == "" {
        output_directory = "./osu songs".to_string();
    }
    let output_path: &Path = Path::new(output_directory.as_str());
    if !output_path.is_dir() {
        fs::create_dir_all(output_path).unwrap()
    }

    let entries = fs::read_dir(songs_path)?
        .map(|res| res.map(|e| e))
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    entries.par_iter().for_each(|entry| {
            visit_dirs(&entry, output_path).unwrap();
    });

    Ok(())
}
