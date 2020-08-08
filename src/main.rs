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
    let result = parse_and_gen(unsafe {std::str::from_utf8_unchecked(&source_code)})?;
    if let Some(out) = cmd_opt.output {
        let output = current_dir()?.join(out);
        OpenOptions::new().create(true).truncate(true)
            .open(output)?
            .write_all(result.as_bytes())?;
    } else {
        println!("{}", result);
    }
    Ok(())
}

fn parse_and_gen(source_code: &str) -> Result<String> {
    let json_value = parse_json(source_code)?;
    let obj = pbgen::visit_json_root(&json_value)?;
    Ok(pbgen::gen_pb_def(&obj))
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
            .help("specify file to save generated proto(default: stdout)")
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
        Ok((_, json_value)) => Ok(json_value),
        Err(nom::Err::Error(e)) => {
            Err(convert_error(source_code, e).into())
        },
        Err(nom::Err::Failure(e)) => {
            Err(convert_error(source_code, e).into())
        },
        Err(nom::Err::Incomplete(_)) => {
            Err("incomplete json".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use nom::lib::std::collections::HashMap;

    #[test]
    fn test_parse_and_gen() -> Result<()> {
        let path = current_dir()?.join("tests/pairs");
        let dirs = fs::read_dir(path)?;
        let mut input = HashMap::new();
        let mut expect = HashMap::new();
        for dir in dirs {
            let dir = dir?;
            let file_name = dir.file_name();
            let f_name = file_name.to_str().unwrap();
            if f_name.ends_with(".json") {
                input.insert(f_name.strip_suffix(".json").unwrap().to_string(), dir.path());
            } else {
                expect.insert(f_name.strip_suffix(".expect").unwrap().to_string(), dir.path());
            }
        }
        for (id, path) in input {
            let mut ifd = OpenOptions::new().read(true).open(path)?;
            let mut source_code = vec![];
            ifd.read_to_end(&mut source_code)?;
            let actual = parse_and_gen(unsafe {std::str::from_utf8_unchecked(&source_code)})?;
            let expect_path = expect.get(&id).unwrap();
            let mut efd = OpenOptions::new().read(true).open(expect_path)?;
            let mut expect_data = vec![];
            efd.read_to_end(&mut expect_data)?;
            assert_eq!(actual, std::str::from_utf8(&expect_data)?);
        }
        Ok(())
    }
}