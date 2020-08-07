use clap::{App, Arg};
use nom::error::{
    VerboseError,
    convert_error,
};
use std::{
    env::current_dir,
    path::Path,
};
use std::fs::OpenOptions;
use std::io::{Read, Write};

mod parser;
mod pbgen;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + 'static>>;

fn main() -> Result<()> {
    let cmd_opt = cmd_args();
    let source_code = read_file(&cmd_opt.input)?;
    let json_value = parse_json(unsafe {std::str::from_utf8_unchecked(&source_code)})?;
    let obj = pbgen::visit_json_root(&json_value)?;
    let result = pbgen::gen_pb_def(&obj);
    if let Some(out) = cmd_opt.output {
        let output = current_dir()?.join(out);
        OpenOptions::new().create(true).truncate(true)
            .open(output)?
            .write_all(result.as_bytes());
    } else {
        println!("{}", result);
    }
    Ok(())
}

struct CmdOpt {
    input: String,
    output: Option<String>,
}

fn read_file<P: AsRef<Path>>(p: P) -> Result<Vec<u8>> {
    let cdr = dbg!(current_dir()?);
    let full_path = dbg!(cdr.join(p));
    let mut fd = OpenOptions::new().read(true).open(full_path)?;
    let mut data = vec![];
    fd.read_to_end(&mut data)?;
    Ok(data)
}

fn cmd_args() -> CmdOpt {
    let matches = App::new("json2pb")
        .version("0.1.0")
        .about("convert json to protobuf3")
        .author("ronaldoliu@tencent.com")
        .arg(Arg::with_name("file")
            .required(true)
            .takes_value(true)
            .short("f")
            .help("specify json file name")
        )
        .arg(Arg::with_name("out")
            .required(false)
            .takes_value(true)
            .short("o")
            .help("specify file to save generated proto")
        )
        .get_matches();
    let input = matches.value_of("file").unwrap().to_string();
    let output = matches.value_of("out").map(|v| v.to_string());
    CmdOpt{
        input,
        output,
    }
}

fn parse_json(source_code: &str) -> Result<parser::JsonValue> {
    match parser::root::<VerboseError<&str>>(source_code) {
        Ok((_, jsonValue)) => Ok(jsonValue),
        Err(nom::Err::Error(e)) => {
            Err(convert_error(source_code, e).into())
        },
        Err(nom::Err::Failure(e)) => {
            Err(convert_error(source_code, e).into())
        },
        Err(nom::Err::Incomplete(v)) => {
            Err("incomplete json".into())
        }
    }
}