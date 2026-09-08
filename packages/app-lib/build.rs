use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, exit};
use std::{env, fs};

fn main() {
    println!("cargo::rerun-if-changed=.env");
    println!("cargo::rerun-if-env-changed=CURSEFORGE_API_KEY");
    println!("cargo::rerun-if-changed=java/gradle");
    println!("cargo::rerun-if-changed=java/src");
    println!("cargo::rerun-if-changed=java/build.gradle.kts");
    println!("cargo::rerun-if-changed=java/settings.gradle.kts");
    println!("cargo::rerun-if-changed=java/gradle.properties");

    #[cfg(target_os = "windows")]
    if env::var_os("CARGO_FEATURE_TAURI").is_some() {
        println!(
            "cargo::rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
        );
    }

    set_env();
    build_java_jars();
}

fn set_env() {
    let curseforge_api_key = env::var("CURSEFORGE_API_KEY")
        .ok()
        .or_else(|| read_dotenv_literal("CURSEFORGE_API_KEY"));

    for (var_name, var_value) in
        dotenvy::dotenv_iter().into_iter().flatten().flatten()
    {
        if var_name == "DATABASE_URL" || var_name == "CURSEFORGE_API_KEY" {
            // The sqlx database URL is a build-time detail that should not be exposed to the crate
            continue;
        }

        println!("cargo::rustc-env={var_name}={var_value}");
    }

    if let Some(curseforge_api_key) = curseforge_api_key {
        println!("cargo::rustc-env=CURSEFORGE_API_KEY={curseforge_api_key}");
    }
}

fn read_dotenv_literal(name: &str) -> Option<String> {
    let contents = fs::read_to_string(".env").ok()?;

    contents.lines().find_map(|line| {
        let line = line.trim_start().strip_prefix("export ").unwrap_or(line);
        let (candidate, value) = line.split_once('=')?;
        if candidate.trim() != name {
            return None;
        }

        let value = value.trim();
        let value = if value.len() >= 2
            && ((value.starts_with('\'') && value.ends_with('\''))
                || (value.starts_with('"') && value.ends_with('"')))
        {
            &value[1..value.len() - 1]
        } else {
            value
        };

        (!value.is_empty()).then(|| value.to_string())
    })
}

fn build_java_jars() {
    let out_dir =
        dunce::canonicalize(PathBuf::from(env::var_os("OUT_DIR").unwrap()))
            .unwrap();

    println!(
        "cargo::rustc-env=JAVA_JARS_DIR={}",
        out_dir.join("java/libs").display()
    );

    let gradle_path = fs::canonicalize(
        #[cfg(target_os = "windows")]
        "java\\gradlew.bat",
        #[cfg(not(target_os = "windows"))]
        "java/gradlew",
    )
    .unwrap();

    let mut build_dir_str = OsString::from("-Dorg.gradle.project.buildDir=");
    build_dir_str.push(out_dir.join("java"));
    let exit_status = Command::new(gradle_path)
        .arg(build_dir_str)
        .arg("build")
        .arg("--no-daemon")
        .arg("--console=rich")
        .current_dir(dunce::canonicalize("java").unwrap())
        .status()
        .expect("Failed to wait on Gradle build");

    if !exit_status.success() {
        println!("cargo::error=Gradle build failed with {exit_status}");
        exit(exit_status.code().unwrap_or(1));
    }
}
