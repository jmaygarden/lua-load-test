use clap::StructOpt;
use std::{io::Read, time::Duration};
use zip_parser::find_local_file;

#[derive(clap::Parser)]
struct Args {
    #[clap(subcommand)]
    action: Action,
}

#[derive(clap::Subcommand)]
enum Action {
    File,
    Zip,
    ZipParser,
}

enum Error {
    DecompressError(flate2::DecompressError),
    GenericError(&'static str),
    IoError(std::io::Error),
    MLuaError(mlua::Error),
    ZipError(zip::result::ZipError),
    ZipParserError(zip_parser::Error),
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DecompressError(inner) => write!(f, "Decompress Error: {:?}", inner),
            Error::GenericError(inner) => write!(f, "String Error: {}", *inner),
            Error::IoError(inner) => write!(f, "IO Error: {:?}", inner),
            Error::MLuaError(inner) => write!(f, "Lua Error: {:?}", inner),
            Error::ZipError(inner) => write!(f, "ZIP Error: {:?}", inner),
            Error::ZipParserError(inner) => write!(f, "ZIP Parser Error: {:?}", inner),
        }
    }
}

impl From<flate2::DecompressError> for Error {
    fn from(inner: flate2::DecompressError) -> Self {
        Self::DecompressError(inner)
    }
}

impl From<&'static str> for Error {
    fn from(inner: &'static str) -> Self {
        Self::GenericError(inner)
    }
}

impl From<std::io::Error> for Error {
    fn from(inner: std::io::Error) -> Self {
        Self::IoError(inner)
    }
}

impl From<mlua::Error> for Error {
    fn from(inner: mlua::Error) -> Self {
        Self::MLuaError(inner)
    }
}

impl From<zip::result::ZipError> for Error {
    fn from(inner: zip::result::ZipError) -> Self {
        Self::ZipError(inner)
    }
}

impl From<zip_parser::Error> for Error {
    fn from(inner: zip_parser::Error) -> Self {
        Self::ZipParserError(inner)
    }
}

type Result<T> = std::result::Result<T, Error>;

fn mock(lua: &mlua::Lua) -> mlua::Result<mlua::Table> {
    let exports = lua.create_table()?;
    let print = lua.create_function(|_lua, args: mlua::MultiValue| {
        let mut iter = args.into_iter();

        if let Some(value) = iter.next() {
            println!("{:?}", value);
        }

        while let Some(value) = iter.next() {
            println!("\t{:?}", value);
        }

        Ok(())
    })?;

    exports.set("print", print)?;

    Ok(exports)
}

async fn stat() {
    tokio::time::sleep(Duration::from_millis(1)).await;

    let pid = procfs::process::Process::myself().unwrap();
    let smaps = pid.smaps().unwrap();
    let mut total = None;

    for (map, data) in smaps.into_iter() {
        if map.pathname == procfs::process::MMapPath::Heap {
            let size = data.map.get("Rss").map(|value| *value);

            total = total.map_or(size, |total: u64| size.map(|value| total + value));
        }
    }

    println!("heap {}", total.unwrap());
}

const FILE_PATH: &str = "lua/init.lua";

async fn run_chunk<'a>(
    lua: &'a mlua::Lua,
    chunk: Vec<u8>,
) -> Result<mlua::MultiValue<'a>> {
    println!("run chunk");
    let module: mlua::Table = lua
        .load(&chunk)
        .set_name(FILE_PATH)?
        .set_mode(mlua::ChunkMode::Text)
        .call_async(())
        .await?;
    let main: mlua::Function = module.get("main")?;
    stat().await;

    let result = main.call_async(()).await.map_err(|error| error.into());
    println!("done");
    result
}

async fn load_file() -> Result<Vec<u8>> {
    println!("load_file");
    let mut file = std::fs::File::open(FILE_PATH)?;
    let mut chunk = Vec::new();

    file.read_to_end(&mut chunk)?;
    stat().await;

    Ok(chunk)
}

async fn load_zip() -> Result<Vec<u8>> {
    println!("load_zip");
    const ZIP_PATH: &str = "lua.zip";
    let file = std::fs::File::open(ZIP_PATH)?;
    let mut zip = zip::ZipArchive::new(file)?;
    let mut file = zip.by_name(FILE_PATH)?;
    let mut chunk = Vec::new();

    file.read_to_end(&mut chunk)?;
    stat().await;

    Ok(chunk)
}

async fn load_zip_parser() -> Result<Vec<u8>> {
    println!("load_zip_parser");
    const ZIP_PATH: &str = "lua.zip";
    let mut file = std::fs::File::open(ZIP_PATH)?;
    let local_file = find_local_file(&mut file, FILE_PATH)?;
    let (_reader, chunk) = local_file.extract_uncompressed()?;

    stat().await;

    Ok(chunk)
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let lua = {
        let lua = mlua::Lua::new();

        lua.globals().set("mock", mock(&lua).unwrap()).unwrap();

        lua
    };
    let args = Args::parse();

    stat().await;

    match args.action {
        Action::File => {
            let chunk = load_file().await.unwrap();
            stat().await;
            let result = run_chunk(&lua, chunk).await;
            println!("run_file => {:?}", result);
        }
        Action::Zip => {
            let chunk = load_zip().await.unwrap();
            stat().await;
            let result = run_chunk(&lua, chunk).await;
            println!("run_zip => {:?}", result);
        }
        Action::ZipParser => {
            let chunk = load_zip_parser().await.unwrap();
            stat().await;
            let result = run_chunk(&lua, chunk).await;
            println!("run_zip_parser => {:?}", result);
        }
    }

    stat().await;
}

mod zip_parser;
