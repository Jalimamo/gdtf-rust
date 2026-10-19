use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::collections::HashMap;
use std::error::Error;
use std::fs::OpenOptions;
use crate::gdtf_xml_scheme::Gdtf;
use reqwest::blocking::Client;
use serde_json::Value;
use std::io::{Cursor, Read};
use std::thread;
use std::time::Duration;
use zip::ZipArchive;
use parsing_helpers::verification::{Verify, VerificationRoots};

mod parsing_helpers;
mod gdtf_xml_scheme;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_gdtf_api_stress_test()
}

fn _parse_tmp_gdtf_folder() -> Result<(), Box<dyn Error>> {
    let xml = std::fs::read_to_string("/tmp/gdtf/description.xml")?;

    // 1. Erzeuge den Deserializer manuell
    let mut deserializer = quick_xml::de::Deserializer::from_str(&xml);

    // 2. Wende den Path-Tracker an
    let gdtf: Gdtf = match serde_path_to_error::deserialize(&mut deserializer) {
        Ok(parsed) => parsed,
        Err(err) => {
            eprintln!("❌ PARSER-FEHLER!");
            eprintln!("📍 Exakter XML-Pfad: {}", err.path());
            eprintln!("💥 Grund: {}", err.into_inner());
            return Err("Deserialisierung fehlgeschlagen".into());
        }
    };

    println!("Hello, world!");
    println!("{:?}", gdtf);
    Ok(())
}

pub fn run_gdtf_api_stress_test() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Client mit Cookie-Store initialisieren
    let client = Client::builder()
        .cookie_store(true)
        .timeout(Duration::from_secs(15)) // Timeout hinzugefügt, damit er bei Server-Lags nicht ewig hängt
        .build()?;

    perform_login(&client)?;

    // 3. Fixture-Liste abrufen
    // (Den exakten Dateinamen für GetList in der API.md eintragen)
    let fixtures_res: Value = client.post("https://gdtf-share.com/apis/public/getList.php")
        .json(&serde_json::json!({}))
        .send()?
        .json()?;

    // println!("📦 API JSON-Antwort: {:#?}", fixtures_res);

    let fixtures = match fixtures_res.get("list").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => {
            // Falls es kein direkter Array ist, prüfen wir typische Wrapper-Keys (z.B. ein Objekt mit "data" oder "fixtures")
            if let Some(obj) = fixtures_res.as_object() {
                eprintln!("⚠️ API hat ein Objekt statt eines Arrays geliefert. Vorhandene Keys: {:?}", obj.keys());
            }
            return Err("Die JSON-Struktur entspricht nicht der Erwartung (kein Root-Array).".into());
        }
    };

    //let fixtures = fixtures_res["fixtures"].as_array().expect("Erwartete JSON-Array");

    println!("Got {} Fixtures", fixtures.len());

    let mut fixture_count= 0;
    let mut fixtures_downloaded = 0;
    let mut fixtures_unzipped = 0;
    let mut correct_compiled = 0;
    let mut verified = 0;

    let mut parse_errors: HashMap<String, (usize, Vec<String>)> = HashMap::new();
    let mut verify_warnings: HashMap<usize, Vec<String>> = HashMap::new();
    let mut multiple_logical_channel_rids: Vec<u64> = Vec::new();

    // Wir begrenzen den Testlauf, um keinen IP-Bann zu riskieren
    for fixture in fixtures.iter() {
        // Die API liefert unter anderem die Revision ID (rid) und den Fixture-Namen
        let rid = match fixture.get("rid").and_then(|v| v.as_u64()) {
            Some(id) => id,
            None => continue, // Überspringen, falls rid fehlt oder defekt ist
        };
        if rid != 163307 {
            continue;
        }
        let name = fixture.get("fixture").and_then(|v| v.as_str()).unwrap_or("Unknown");
        let manufacturer = fixture.get("manufacturer").and_then(|v| v.as_str()).unwrap_or("Unknown");
        let revision = fixture.get("revision").and_then(|v| v.as_str()).unwrap_or("Unknown");

        fixture_count += 1;

        let mut retry_count = 0;

        let bytes = loop {
            thread::sleep(Duration::from_millis(300));

            let download_res = match client.get("https://gdtf-share.com/apis/public/downloadFile.php")
                .query(&[("rid", rid)])
                .send()
            {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("\r\x1b[2K❌ Netzwerkfehler bei RID {}: {}", rid, e);
                    break None; // Bricht den Loop ab und liefert None
                }
            };

            let b = match download_res.bytes() {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("\r\x1b[2K❌ Fehler beim Lesen der Bytes bei RID {}: {}", rid, e);
                    break None;
                }
            };

            if b.len() < 2 || &b[0..2] != b"PK" {
                let error_text = String::from_utf8_lossy(&b);

                // Falls es nach Auth-Fehler riecht und wir noch nicht re-tried haben
                if retry_count < 1 && (error_text.contains("Session") || error_text.contains("error") || error_text.contains("false")) {
                    print!("\r\x1b[2K⚠️ Session abgelaufen? Versuche Re-Login...");
                    std::io::Write::flush(&mut std::io::stdout())?;

                    if let Err(e) = perform_login(&client) {
                        eprintln!("\n❌ Re-Login fehlgeschlagen: {}", e);
                    }
                    retry_count += 1;
                    continue; // Loop wiederholen und Download neu versuchen
                } else {
                    eprintln!("\r\x1b[2K❌ API hat kein ZIP gesendet für {} (RID: {}). Antwort war: {}", name, rid, error_text);
                    break None;
                }
            }

            break Some(b)
        };

        let bytes = match bytes {
            Some(bytes) => {
                fixtures_downloaded += 1;
                bytes
            },
            None => continue,
        };

        let mut archive = match ZipArchive::new(Cursor::new(bytes)) {
            Ok(zip) => zip,
            Err(_) => {
                eprintln!("\r\x1b[2K❌ Ungültiges ZIP-Archiv bei {}", name);
                continue;
            }
        };

        let mut xml_data = String::new();

        {
            let mut xml_file = match archive.by_name("description.xml") {
            Ok(file) => file,
                Err(_) => {
                    eprintln!("\r\x1b[2K❌ description.xml fehlt in RID {}", rid);
                    continue;
                }
            };

            if let Err(e) = xml_file.read_to_string(&mut xml_data) {
                eprintln!("\r\x1b[2K❌ Lesefehler der XML in RID {}: {}", rid, e);
                continue;
            }

        }

        fixtures_unzipped += 1;

        let url = format!("https://fixturebuilder.gdtf-share.com/load/?rid={}", rid);

        print!("\r\x1b[2K Deserialising Nr {}: {}@{}@{} (RID: {})", fixture_count, manufacturer, name, revision, rid);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        // 6. Den Serde-Parser draufjagen
        let mut deserializer = quick_xml::de::Deserializer::from_str(&xml_data);
        match serde_path_to_error::deserialize::<_, Gdtf>(&mut deserializer) {
            Ok(mut gdtf) => {
                print!("\r\x1b[2K Verifying {}: {}@{}@{} (RID: {})", fixture_count, manufacturer, name, revision, rid);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
                correct_compiled += 1;
                let mut roots = VerificationRoots::new();
                match gdtf.verify(&mut roots, &mut archive) {
                    Ok(warnings) => {
                        let entry = verify_warnings
                            .entry(rid as usize)
                            .or_insert_with(Vec::new);

                        if warnings.iter().any(|warning| warning == "Ich hab mehrere LogicalChannel") {
                        multiple_logical_channel_rids.push(rid);
                        }

                        if warnings.len() > 0 {
                            entry.extend(warnings);
                        }

                        verified += 1;
                        print!(
                            "\r\x1b[2K✅ Nr {} Erfolgreich: {}@{}@{} (RID: {})",
                            fixture_count, manufacturer, name, revision, rid
                        );
                        std::io::Write::flush(&mut std::io::stdout()).unwrap();
                    }
                    Err(e) => {
                        let entry = parse_errors
                            .entry("Verification-error".into())
                            .or_insert((0, Vec::new()));
                        entry.0 += 1;
                        entry.1.push(format!("❌ Verifier-CRASH bei {}@{}@{}, ({})\n Grund: {}",
                                             manufacturer, name, revision, url, e));
                    }
                };
            },
            Err(err) => {
                let err_path = err.path().to_string();
                let xml_context = get_xml_error_context(&mut xml_data, err_path);

                let full_error = format!(
                    "❌ PARSER-CRASH bei {}@{}@{}, ({})\n📍 Pfad: {}\n💥 Grund: {}\n📄 XML-Kontext um das Problem:\n{}",
                     manufacturer, name, revision, url, err.path(), err.clone().into_inner(), xml_context);
                // println!();
                // println!();
                // eprintln!("{}", full_error);
                // println!();
                // println!();

                let entry = parse_errors
                    .entry(err.clone().into_inner().to_string())
                    .or_insert((0, Vec::new()));
                entry.0 += 1;
                entry.1.push(full_error);
            }
        }
    }

    println!("\n\n📊 TESTABSCHLUSS:");
    println!("Gesamtzahl versuchter Fixtures:       {}", fixture_count);

    let dl_percent = (fixtures_downloaded as f64 / fixture_count as f64) * 100.0;
    println!("📥 Erfolgreich heruntergeladen:        {} ({}%)", fixtures_downloaded, dl_percent);

    let uz_percent = (fixtures_unzipped as f64 / fixtures_downloaded as f64) * 100.0;
    println!("📦 Erfolgreich entpackt:               {} ({}% der Downloads)", fixtures_unzipped, uz_percent);

    let parse_percent = (correct_compiled as f64 / fixtures_unzipped as f64) * 100.0;
    println!("✅ Fehlerfrei geparst:                 {} ({}% der entpackten XMLs)", correct_compiled, parse_percent);

    let verified_percent = (verified as f64 / correct_compiled as f64) * 100.0;
    println!("✅ Erfolgreich verifiziert:            {} ({}% der geparsten GDTFs)", verified, verified_percent);

    let total_percent = (verified as f64 / fixture_count as f64) * 100.0;
    println!("🏆 Gesamterfolgsquote                  {}%\n", total_percent);

    if let Ok(mut log_file) = OpenOptions::new().create(true).append(true).open("parser_errors.log") && !parse_errors.is_empty() {
        writeln!(log_file, "🚨 AGGREGIERTE PARSER-FEHLER (Nach Häufigkeit sortiert):")?;

        // HashMap in einen Vector umwandeln und nach Häufigkeit (absteigend) sortieren
        let mut sorted_errors: Vec<_> = parse_errors.into_iter().collect();
        sorted_errors.sort_by(|a, b| b.1.0.cmp(&a.1.0));

        for (reason, (count, full_errors)) in sorted_errors {
            writeln!(log_file, "  - {}x: {}", count, reason)?;

            // Klickbare URLs für die Beispiele generieren
            writeln!(log_file, "    Beispiele: ")?;
            for (i, full_error) in full_errors.iter().enumerate() {
                write!(log_file, "{}", full_error)?;
                if i < full_errors.len() - 1 { write!(log_file, "\n\n")?; }
            }
            writeln!(log_file, "\n\n\n\n")?;
        }
    }

    if let Ok(mut log_file) = OpenOptions::new().create(true).append(true).open("verify_warnings.log") && !verify_warnings.is_empty() {
        writeln!(log_file, "🚨 AGGREGIERTE PARSER-FEHLER (Nach Häufigkeit sortiert):")?;

        // HashMap in einen Vector umwandeln und nach Häufigkeit (absteigend) sortieren
        let warnings_vec: Vec<_> = verify_warnings.into_iter().collect();

        for (rid, warnings) in warnings_vec {
            writeln!(log_file, "Warnings for RID {} (https://fixturebuilder.gdtf-share.com/load/?rid={})",rid, rid )?;

            // Klickbare URLs für die Beispiele generieren
            for warning in warnings.iter() {
                writeln!(log_file, "{}", warning)?;
            }
            writeln!(log_file, "\n\n\n\n")?;
        }
    }

    let mut logical_channel_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("multiple_logical_channel_rids.txt")?;

    multiple_logical_channel_rids.sort_unstable();
    multiple_logical_channel_rids.dedup();

    for rid in multiple_logical_channel_rids {
        writeln!(logical_channel_file, "{}", rid)?;
    }



    Ok(())
}

fn get_xml_error_context(xml_data: &mut String, err_path: String) -> String {
    let target_tag = err_path.split('.').last().unwrap_or("");

    let mut output = String::new();

    let lines: Vec<&str> = xml_data.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if !target_tag.is_empty() && line.contains(target_tag) {
            let start = i.saturating_sub(3);
            let end = (i + 4).min(lines.len());

            for idx in start..end {
                let marker = if idx == i { "--> " } else { "    " };
                writeln!(output, "{}{:4}: {}", marker, idx + 1, lines[idx]).unwrap();
            }

            break;
        }
    }

    output
}

fn perform_login(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let login_payload = serde_json::json!({
        "user": "Bugmenot",
        "password": "1234567"
    });

    let login_res: Value = client.post("https://gdtf-share.com/apis/public/login.php")
        .json(&login_payload)
        .send()?
        .json()?;

    if !login_res.get("result").and_then(|v| v.as_bool()).unwrap_or(false) {
        return Err("FATAL: API-Login fehlgeschlagen. Credentials prüfen!".into());
    } else {
        if let Some(notice) = login_res.get("notice").and_then(|v| v.as_str()) {
            println!("Login Notice: {}", notice);
        }
    }
    Ok(())
}