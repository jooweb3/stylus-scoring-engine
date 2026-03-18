use rusqlite::{Connection, params};
use shared::{Rule, Direction};
use std::sync::Mutex;

pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    pub fn new(path: &str) -> Self {
        let conn = Connection::open(path).expect("failed to open db");
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS rules (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                factor_name TEXT NOT NULL,
                weight INTEGER NOT NULL,
                threshold INTEGER NOT NULL,
                direction TEXT NOT NULL,
                penalty_or_bonus INTEGER NOT NULL
            )"
        ).expect("failed to create table");
        Self { conn: Mutex::new(conn) }
    }

    pub fn insert_rule(&self, rule: &Rule) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().unwrap();
        let dir = match rule.direction {
            Direction::Above => "above",
            Direction::Below => "below",
        };
        conn.execute(
            "INSERT INTO rules (factor_name, weight, threshold, direction, penalty_or_bonus) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![rule.factor_name, rule.weight, rule.threshold, dir, rule.penalty_or_bonus],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_rules(&self) -> rusqlite::Result<Vec<Rule>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT factor_name, weight, threshold, direction, penalty_or_bonus FROM rules")?;
        let rules = stmt.query_map([], |row| {
            let dir_str: String = row.get(3)?;
            let direction = match dir_str.as_str() {
                "above" => Direction::Above,
                _ => Direction::Below,
            };
            Ok(Rule {
                factor_name: row.get(0)?,
                weight: row.get(1)?,
                threshold: row.get(2)?,
                direction,
                penalty_or_bonus: row.get(4)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(rules)
    }
}
