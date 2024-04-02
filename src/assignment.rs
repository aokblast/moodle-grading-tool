use std::{fs::DirEntry, path::Path, io, alloc::System, process::{Command, Stdio}};

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

use crate::Record;

#[derive(Clone, Copy, Debug)]
pub enum ProgramType {
	C
}

impl ProgramType {
	fn get_suffixes(self) -> &'static [(&'static str, &'static str)] {
		match self {
			Self::C => &[("c", "cc")]
		}
	}

	fn grade(self, file_name: &str, dir_name: &str, prog_name: &str) -> Option<u32> {
		let output_file = format!("{dir_name}/output");
		match self {
			ProgramType::C => {
				Command::new(prog_name).arg(file_name).arg(&format!("-o '{output_file}'")).spawn().unwrap();
				Command::new(output_file).spawn().unwrap();
			}
		}

		let mut line = String::new();
		io::stdin().read_line(&mut line).unwrap();
		Some(line.trim().parse().unwrap())
	}
}

#[derive(Clone, Copy, Debug)]
pub enum FileType {
	DOC,
	PROGRAM(ProgramType, bool), // (Which type of program, raw file?)
	PIC
}

impl FileType {
	// Get the (file_extension, command) pair of a program
	fn get_suffixes(self) -> &'static [(&'static str, &'static str)] {
		match self {
			FileType::DOC => {
				&[("txt", "kate"), ("pdf", "evince")]
			}
			FileType::PIC => {
				&[("jpg", "feh"), ("png", "feh")]
			}
			FileType::PROGRAM(program, _) => {
				program.get_suffixes()
			}
		}
	}

	fn grade(self, file_name: &str, dir_name: &str, prog_name: &str) -> Option<u32> {
		if !Path::new(file_name).exists() {
			return None
		}

		match self {
			FileType::DOC => {
				Command::new(prog_name).arg(file_name).stdin(Stdio::null()).stdout(Stdio::null()).spawn().unwrap();
			}
			FileType::PIC => {
				Command::new(prog_name).arg(file_name).stdin(Stdio::null()).stdout(Stdio::null()).spawn().unwrap();
			}
			FileType::PROGRAM(program, is_raw) => {
				if is_raw {
					Command::new("kate").stdin(Stdio::null()).stdout(Stdio::null()).arg(file_name).spawn().unwrap();
				} else {
					return program.grade(file_name, dir_name, prog_name);
				}
			}
		}

		println!("Type the score(0-100):");
		// Read the score
		let mut line = String::new();
		io::stdin().read_line(&mut line).unwrap();
		Some(line.trim().parse().unwrap_or(0))
	}

}


// I use system in the rust code. To prevent any security issue, I decide to allow only string literal as problem_name
#[derive(Clone)]
pub struct Problem {
	problem_name: &'static str,
	file_type: FileType,
	pub percentage: u32,
}


impl Problem {
	pub fn new(problem_name: &'static str, file_type: FileType, percentage: u32) -> Self {
		Self {
			problem_name,
			file_type,
			percentage
		}
	}

	// Per problem grade
	pub fn grade(&self, dir: &str) -> u32 {
		// Find possible suffixes
		let suffixes = self.file_type.get_suffixes();

		for suffix in suffixes {
			// Get true file name and search each
			println!("Searching {}.{}", self.problem_name, suffix.0);
			let file_name = format!("{dir}/{}.{}", self.problem_name, suffix.0);

			if let Some(score) = self.file_type.grade(&file_name, &dir, suffix.1) {
				return score
			}
		}
		
		101
	}
}

pub struct Assignment {
	problems: Vec<Problem>,
}

impl Assignment {
	pub fn new() -> Self {
		Self {
			problems: vec![]
		}
	}
	pub fn new_with_problems(problems: &[Problem]) -> Self {
		let mut res = Self {
			problems: vec![]
		};

		for problem in problems {
			res.problems.push(problem.clone());
		}

		res
	}

	pub fn add_entry(&mut self, problem: Problem) {
		self.problems.push(problem);
	}

	pub fn get_comment() -> String {
		let mut line = String::new();
		io::stdin().read_line(&mut line).unwrap();
		line = line.trim().to_string();
		line
	}

	// Read from dir, get the (score, comment) of an assignment
	pub fn grade(&self, dir_name: &str, student_id: &str) -> (f64, String) {
		let (mut scores, mut total_percentage) = (0.0, 0.0);
		let mut comment = String::new();

		let mut dir_name = dir_name.to_string();

		// Get inner directory
		if Path::new(&format!("{}/{}", dir_name, student_id.to_uppercase())).exists() {
			dir_name += "/";
			dir_name += &student_id.to_uppercase();
		} else if Path::new(&format!("{}/{}", dir_name, student_id.to_lowercase())).exists() {
			dir_name += "/";
			dir_name += &student_id.to_lowercase();
		}

		let mut need_reviews = false;
		// Grade all problems
		for problem in &self.problems {
			// Grade scores
			println!("Grading problem {} on student: {student_id}", problem.problem_name);
			
			let score = problem.grade(&dir_name);
			if score != 101 {
				scores += score as f64 * problem.percentage as f64;
			} else {
				need_reviews = true;
			}


			// Get comment
			println!("Write down your comment:");
			let cur_comment = Self::get_comment();
			if cur_comment.len() != 0 {
				comment +=  &format!("{}: {}.", problem.problem_name, cur_comment);
			}

			total_percentage += problem.percentage as f64;
		}

		(scores / total_percentage, comment + if need_reviews {"Need manual review"} else {""})
	}
}

