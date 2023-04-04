use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;
use std::path::PathBuf;

use clap::Parser;

use manifest::AppLabel;

use minijinja::{context, Environment};
use serde::Serialize;

#[derive(Serialize)]
pub struct Context {
    name: String,
}

fn read_app_label_from_file<P: AsRef<Path>>(path: P) -> Result<AppLabel, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `AppLabel`.
    let app_label = serde_json::from_reader(reader)?;

    Ok(app_label)
}

#[derive(Parser, Debug)]
struct Opts {
    /// Path to the JSON file containing the App Label
    #[clap(short, value_parser)]
    app_label_path: PathBuf,

    /// Output path of the generated XACML requests.
    /// Each request file is saved with the name "request_\<x\>.xml",
    /// where \<x\> is an increasing integral starting from 1.
    #[clap(short, value_parser)]
    output_path: Option<PathBuf>,

    /// Enable additional information about the underlying process
    /// and print the generated XACML requests to the standard output.
    #[clap(short, long)]
    verbose: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::parse();

    // check output_path
    // - if no output path was specified, set 'save' to false and continue;
    // - if the output path is not an existing directory, panic.
    // - if the output path is an existing directory, set 'save' to true
    //   and keep the path in 'dir_path' to be used later
    let mut dir_path = PathBuf::new();
    let save = if let Some(val) = opts.output_path {
        if val.is_dir() {
            dir_path = val;
            true
        } else {
            panic!("{:?} is not an existing directory.", val);
        }
    } else {
        false
    };

    let app_label = read_app_label_from_file(opts.app_label_path)?;

    let mut env = Environment::new();
    env.add_template(
        "request.xml",
        include_str!("../../templates/request.xml"
        ))?;

    let tmpl = env.get_template("request.xml")?;

    println!(
        "\n> Creating XACML requests from app: \"{}\"...",
        app_label.app_name
    );

    for idx in 0..app_label.api_labels.len() {
        let req = tmpl.render(context!(
            app_label,
            index => idx,
        )).unwrap();

        if opts.verbose {
            println!("{}", req);
        }

        if save {
            let mut file = File::create(
                dir_path.join(format!("request_{}.xml", idx + 1)))?;
            file.write_all(req.as_ref())?;
        }
    }

    print!("\n> XACML requests created successfully");
    if save {
        println!(" and saved to {:?}", dir_path)
    }
    else {
        print!("\n");
        if !opts.verbose {
            println!("\n> Hint: specify the option -v to print the XACML requests or \
            -o <OUTPUT_PATH> to save the XACML requests in a specific directory.")
        }
    }

    Ok(())
}
