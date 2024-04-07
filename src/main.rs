use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::Stdio,
    result,
};

use assignment::{Assignment, FileType, Problem, ProgramType};
use calamine::{
    open_workbook, open_workbook_auto, Error, RangeDeserializer, RangeDeserializerBuilder, Reader,
    Xlsx,
};
use clap::Parser;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use serde::{de::value::BytesDeserializer, Deserialize, Serialize};

mod assignment;

#[derive(Deserialize, Serialize, Debug)]
struct Record {
    #[serde(rename = "序號(No.)")]
    id: u32, // class generated ID
    #[serde(rename = "學號(Stu No.)")]
    student_number: String, // student number
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long, default_value = "./")]
    working_dir: String,
    #[clap(short, long, default_value = "test.xlsx")]
    file_path: String,
    #[clap(short, long, default_value = "output.txt")]
    output_file: String,
}

fn list_directory(path: &str) -> Vec<String> {
    let files = fs::read_dir(path).unwrap();
    let mut res = vec![];

    for file in files {
        res.push(file.unwrap().path().to_str().unwrap().to_string());
    }

    res
}

fn file_matching(matcher: &SkimMatcherV2, file_name: &str, items: &Vec<String>) -> Option<String> {
    let mut hightest_score = 0;
    let mut res = String::new();

    for item in items {
        if let Some(score) = matcher.fuzzy_match(&item, &file_name) {
            if hightest_score < score {
                hightest_score = score;
                res = item.clone();
            }
        }
    }

    if res.is_empty() {
        None
    } else {
        Some(res)
    }
}

fn grading(
    item: &Record,
    matcher: &SkimMatcherV2,
    dirs: &Vec<String>,
    assignment: &Assignment,
    output_file: &mut File,
) {
    let mut file_name = PathBuf::new();

    println!("Start grading student: {}", &item.student_number);

    // Match with zip file in file_directory
    if let Some(matched_directory) = file_matching(matcher, &item.student_number, dirs) {
        file_name.push(matched_directory);
        file_name.push(item.student_number.to_uppercase());
        file_name.set_extension("zip");

        // Use lowercase
        if !file_name.exists() {
            file_name.pop();
            file_name.push(item.student_number.to_lowercase());
            file_name.set_extension("zip");
        }
    } else {
        println!(
            "Zip file for {} not found. Please type a file name for your zip file:",
            item.student_number
        );
        let mut buf = String::new();
        io::stdin().read_line(&mut buf).unwrap();
        file_name = PathBuf::from(buf);
    }

    let (scores, comment) = if file_name.exists() {
        // Unzip the file
        let workdir = file_name.parent().unwrap().to_str().unwrap();
        println!("Unziping zipfile: {:?} in workdir: {workdir}", file_name);
        std::process::Command::new("unzip")
            .arg(&file_name)
            .arg(&format!("-d{workdir}"))
            .stdout(Stdio::null())
            .status()
            .unwrap();

        // Grade the scores
        assignment.grade(workdir, &item.student_number)
    } else {
        (-1.0, "Need manual review".to_string())
    };

    output_file
        .write_fmt(format_args!("{scores}\t{comment}\n"))
        .unwrap();
    output_file.flush().unwrap();
}

fn count_line(f: File) -> (File, u32) {
    let mut cur = 0;
    let mut reader = BufReader::new(f);
    let mut line = String::new();

    while let Ok(byte) = reader.read_line(&mut line) {
        if byte == 0 {
            break;
        }

        cur += 1;
    }

    (reader.into_inner(), cur)
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let dirs = list_directory(&args.working_dir);

    let mut workbook = open_workbook_auto(args.file_path)?;
    let worksheet = workbook.worksheet_range("成績")?;

    let iterator = RangeDeserializerBuilder::new().from_range(&worksheet)?;
    let matcher = SkimMatcherV2::default();

    // TODO: Design your own assignment
    let mut assignment = Assignment::new();
    assignment.add_entry(Problem::new("1", FileType::Pic, 20));
    assignment.add_entry(Problem::new("1", FileType::Doc, 20));
    assignment.add_entry(Problem::new("2", FileType::Doc, 30));
    assignment.add_entry(Problem::new(
        "3",
        FileType::Program(ProgramType::C, true),
        15,
    ));
    assignment.add_entry(Problem::new("3", FileType::Pic, 15));

    let output_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&args.output_file)
        .unwrap();

    let (mut output_file, lines) = count_line(output_file);

    println!("{lines} lines previously.");

    iterator.skip(lines as usize).for_each(move |item| {
        grading(
            &item.unwrap(),
            &matcher,
            &dirs,
            &assignment,
            &mut output_file,
        )
    });

    Ok(())
}
