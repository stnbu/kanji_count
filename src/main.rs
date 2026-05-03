use glob::glob;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::env;
use std::fs;

fn is_kanji(c: char) -> bool {
    matches!(c as u32, 0x4E00..=0x9FFF)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: kanji_counter <glob> [<glob> ...]");
        std::process::exit(1);
    }

    let mut counts: HashMap<char, u64> = HashMap::new();

    for pattern in args {
        for entry in glob(&pattern)? {
            let path = entry?;
            let content = fs::read_to_string(&path)?;
            for c in content.chars() {
                if is_kanji(c) {
                    *counts.entry(c).or_insert(0) += 1;
                }
            }
        }
    }

    let mut conn = Connection::open("kanji.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS kanji (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            char TEXT UNIQUE NOT NULL,
            count INTEGER NOT NULL DEFAULT 0
        )",
        [],
    )?;

    let tx = conn.transaction()?;

    {
        let mut stmt = tx.prepare(
            "INSERT INTO kanji (char, count)
             VALUES (?1, ?2)
             ON CONFLICT(char) DO UPDATE SET count = count + excluded.count",
        )?;

        for (ch, count) in counts {
            let s = ch.to_string();
            stmt.execute(params![s, count as i64])?;
        }
    }

    tx.commit()?;

    Ok(())
}
