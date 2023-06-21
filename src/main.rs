use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;
use std::path::PathBuf;

use clap::Parser;

use manifest::{ApiLabel, AppLabel};

use minijinja::{context, Environment, Template};

static TEMPLATE_FILE: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/", "request.xml"));

#[derive(Parser, Debug)]
struct Opts {
    /// Path to the JSON file containing the App Label
    #[clap(short, value_hint = clap::ValueHint::FilePath)]
    app_label_path: PathBuf,

    /// Output path of the generated XACML requests.
    /// Each request file is saved with the name "request_\<x\>.xml",
    /// where \<x\> is an increasing integral starting from 1.
    #[clap(short, value_hint = clap::ValueHint::DirPath)]
    output_path: Option<PathBuf>,

    /// Enable additional information about the underlying process
    /// and print the generated XACML requests to the standard output.
    ///
    /// Hint: specify the option -v to print the XACML requests or
    /// -o <OUTPUT_PATH> to save the XACML requests in a specific directory.
    #[clap(short, long)]
    verbose: bool,
}
fn read_app_label_from_file<P: AsRef<Path>>(path: P) -> Result<AppLabel, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `AppLabel`.
    serde_json::from_reader(reader).map_err(|e| e.into())
}

#[inline(always)]
fn create_request(
    api_label: &ApiLabel,
    app_name: &str,
    template: Template,
    verbose: bool,
) -> Result<String, Box<dyn Error>> {
    let req = template.render(context!(app_name, api_label))?;

    if verbose {
        println!("{}", req);
    }

    Ok(req)
}

fn create_requests(opts: &Opts) -> Result<(), Box<dyn Error>> {
    let (env, app_label) = deserialize_app_label_and_setup_env(&opts.app_label_path)?;
    let tmpl = env.get_template("request.xml")?;

    for api_label in app_label.api_labels.iter() {
        create_request(api_label, &app_label.app_name, tmpl, opts.verbose)?;
    }

    println!("\n> XACML requests created successfully");

    Ok(())
}

fn create_requests_and_save<P: AsRef<Path>>(
    opts: &Opts,
    dir_path: P,
) -> Result<(), Box<dyn Error>> {
    let (env, app_label) = deserialize_app_label_and_setup_env(&opts.app_label_path)?;
    let tmpl = env.get_template("request.xml")?;

    for (idx, api_label) in app_label.api_labels.iter().enumerate() {
        let req = create_request(api_label, &app_label.app_name, tmpl, opts.verbose)?;

        let mut file = File::create(dir_path.as_ref().join(format!("request_{}.xml", idx + 1)))?;
        file.write_all(req.as_bytes())?;
    }

    println!(
        "\n> XACML requests created successfully and saved to: {}",
        dir_path
            .as_ref()
            .as_os_str()
            .to_str()
            .unwrap_or("Got an error")
    );

    Ok(())
}

fn camel_to_kebab_case(input: &str) -> String {
    let mut kebab_case = String::new();
    let mut prev_char: Option<char> = None;

    for current_char in input.chars() {
        if let Some(prev) = prev_char {
            if prev.is_ascii_lowercase() && current_char.is_ascii_uppercase() {
                kebab_case.push('-');
            }
        }

        kebab_case.push(current_char.to_ascii_lowercase());
        prev_char = Some(current_char);
    }

    kebab_case
}

fn deserialize_app_label_and_setup_env<P: AsRef<Path>>(
    app_label_path: P,
) -> Result<(Environment<'static>, AppLabel), Box<dyn Error>> {
    let app_label = read_app_label_from_file(app_label_path)?;

    let mut env = Environment::new();
    env.add_filter("kebab_case", camel_to_kebab_case);
    env.add_template("request.xml", TEMPLATE_FILE)?;

    println!(
        "\n> Creating XACML requests from app: \"{}\"...",
        app_label.app_name
    );

    Ok((env, app_label))
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::parse();

    if let Some(ref val) = opts.output_path {
        if val.is_dir() {
            create_requests_and_save(&opts, val)?;
        } else {
            panic!(
                "{}: not an existing directory.",
                val.as_os_str().to_str().unwrap_or("Got an error")
            );
        }
    } else {
        create_requests(&opts)?;
    }

    Ok(())
}
