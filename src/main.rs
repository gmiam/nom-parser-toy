use std::fs::{File, OpenOptions};
use anyhow::Error;
use nom::bytes::complete::{take_till, take_until};
use nom::sequence::Tuple;
use nom::IResult;
// use std::fs::read_to_string;
use std::io::{BufReader, read_to_string, Write};
use std::time::Instant;

fn until_semicolon(s: &str) -> IResult<&str, &str> {
    take_until(";")(s)
}
fn till_newline(s: &str) -> IResult<&str, &str> {
    take_till(|c: char| c == '\n')(s)
}
fn gobble_newline(i: &str) -> IResult<&str, char> {
    nom::character::complete::char('\n')(i)
}

fn main() -> Result<(), Error> {
    let start = Instant::now();
    let f = File::open("/Users/guillaume.mazollier/Downloads/000000")?;
    let reader = BufReader::new(f);
    let content = read_to_string(reader)?;
    let mut text = content.as_str();

    let mut output_file = OpenOptions::new().create(true)
        .append(true)
        .open("/tmp/output.sql")?;

    // Requalify parser names for clarity in this scope
    let full_line = till_newline;
    let query_line = until_semicolon;
    let newline = gobble_newline;

    // `from_fn` (available from Rust 1.34) can create an iterator from a closure
    let parser_iterator = std::iter::from_fn(move || {
        match (full_line, newline, full_line, newline, full_line, newline, full_line, newline, query_line,)
            .parse(text)
        {
            // when successful, a nom parser returns a tuple of
            // the remaining input and the output value.
            // So we replace the captured input data with the
            // remaining input, to be parsed on the next call
            Ok((i, o)) => {
                //text = i;
                let (rest, (_, _)) = (full_line, newline).parse(i).ok()?; // remove the remaining semicolon and newline -- the semicolon need to be add back to query later
                text = rest;
                Some(o)
            }
            _ => None,
        }
    });

    let mut i = 0;
    for value in parser_iterator {
        // We match lines from the parsers with the object they represent (5 "lines" -> 5 objects)
        let (_time, _, _user, _, _query_time, _, _timestamp, _, query) = value;
        let query = format!("{};", query.replace("`",""));
        let newline = vec![b'\n'];
        output_file.write_all(query.as_bytes())?;
        output_file.write_all(&newline)?;
        i+=1;
    }

    let duration = start.elapsed();
    println!("***");
    println!("Total number of requests {i}");
    println!("***");
    println!("Time elapsed: {}µs", duration.as_micros());
    Ok(())
}
