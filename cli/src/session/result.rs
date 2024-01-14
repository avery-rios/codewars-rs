use std::fmt::{self, Display, Write};
use yansi::Paint;

use codewars_unofficial::project::result::*;

const IDENT: u8 = 2;

struct ShowOutput<'a>(&'a Output);
fn show_output(output: &Output, prefix: u8, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for _ in 0..prefix {
        f.write_char(' ')?;
    }
    match output {
        Output::Describe { pass: _, v, items } => {
            write!(f, "{}", v)?;
            show_output_slice(items.as_slice(), prefix + IDENT, f)
        }
        Output::It { pass, v, items } => {
            write!(
                f,
                "{} [{}]",
                v,
                if *pass {
                    Paint::green("✔")
                } else {
                    Paint::red("✘")
                }
            )?;
            show_output_slice(items.as_slice(), prefix + IDENT, f)
        }
        Output::CompletedIn { v } => {
            write!(f, "🕑 completed in {} ms", v)
        }
        Output::Passed { v } => {
            write!(f, "{}", Paint::green(format_args!("✔ {}", v)))
        }
        Output::Error { v } => {
            write!(f, "{}", Paint::red(format_args!("✘ {}", v)))
        }
        Output::Failed { v } => {
            write!(f, "{}", Paint::red(format_args!("✘ {}", v)))
        }
    }
}

/// add newline before each output, add nothing if empty
fn show_output_slice(outputs: &[Output], prefix: u8, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if !outputs.is_empty() {
        for o in outputs {
            writeln!(f)?;
            show_output(o, prefix, f)?;
        }
    }
    Ok(())
}
impl<'a> Display for ShowOutput<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        show_output(self.0, IDENT, f)
    }
}

fn show_block(name: &str, content: &Option<String>) {
    if let Some(c) = content {
        println!("{}:", name);
        for l in c.lines() {
            println!("  {}", l);
        }
        println!();
    }
}

pub fn show(test_name: &str, r: &TestResult) {
    println!("{}", test_name);
    r.result
        .output
        .iter()
        .for_each(|o| println!("{}", ShowOutput(o)));
    println!();

    show_block("message", &r.message);
    show_block("stderr", &r.stderr);
    show_block("stdout", &r.stdout);

    print!("Run time: {} ms wall time", r.result.wall_time);
    match r.result.test_time {
        Some(tt) => println!(", {} ms test time", tt),
        None => println!(),
    }
    println!("Exit code: {}", r.exit_code);

    println!(
        "Test result: {}. {}",
        if r.result.timed_out {
            Paint::red("TIME OUT")
        } else if r.exit_code != 0 {
            Paint::red("FAIL")
        } else {
            Paint::green("pass")
        },
        Paint::new(format_args!(
            "{} passed, {} failures, {} errors",
            r.result.passed, r.result.failed, r.result.errors
        ))
        .fg(if r.exit_code == 0 {
            yansi::Color::Green
        } else {
            yansi::Color::Red
        })
    );
}
