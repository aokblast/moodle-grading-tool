use std::{result, collections::HashSet, fs, io, path::Path, process::{Command, Stdio}};

use assignment::{Assignment, Problem};
use calamine::{open_workbook_auto, Error, Reader, RangeDeserializer, RangeDeserializerBuilder, Xlsx, open_workbook};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use serde::{Deserialize, Serialize};
use clap::Parser;

mod assignment;

#[derive(Deserialize, Serialize, Debug)]
struct Record {
	#[serde(rename="序號(No.)")]
	id: u32, // class generated ID
	#[serde(rename="學號(Stu No.)")]
	student_number: String, // student number
	// TODO: Please add the score index of homework
}

#[derive(Parser, Debug)]
struct Args {
	#[clap(short, long, default_value = "./")]
	working_dir: String,
	#[clap(short, long, default_value = "test.xlsx")]
	file_path: String,
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

fn grading(item: &mut Record, matcher: &SkimMatcherV2, files: &Vec<String>, assignment: &Assignment) {
	let mut file_name = String::new();
	let zip_name = &format!("/{}.zip", &item.student_number);
	
	// Match with zip file in file_directory
	if let Some(matched_file) = file_matching(matcher, &item.student_number, files) {
		file_name = matched_file;
		file_name += zip_name;
	} else {
		eprintln!("File {} not found. Please type a file name", zip_name);
		io::stdin().read_line(&mut file_name).unwrap();
	}

	if Path::new(&file_name).exists() {
		let workdir = Path::new(&file_name).parent().unwrap().to_str().unwrap();
		println!("Unziping zipfile: {} in workdir: {}", file_name, workdir);
		Command::new("unzip").arg(&file_name).arg(&format!("-d{}", workdir)).stdout(Stdio::null()).spawn().unwrap();
		assignment.grade(workdir);
	}
}

fn main() -> Result<(), Error> {
	let args = Args::parse();

	let files = list_directory(&args.working_dir);

	let mut workbook = open_workbook_auto(args.file_path)?;
	let worksheet = workbook.worksheet_range("成績")?;

	let iterator = RangeDeserializerBuilder::new().from_range(&worksheet)?;
	let matcher = SkimMatcherV2::default();

	let mut assignment = Assignment::new();

	assignment.add_entry(Problem::new("1", assignment::FileType::PIC, 20));
	assignment.add_entry(Problem::new("1", assignment::FileType::DOC, 20));

	iterator.for_each(move |item| grading(&mut item.unwrap(), &matcher, &files, &assignment));

	Ok(())
}
