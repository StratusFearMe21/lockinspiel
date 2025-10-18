use std::path::Path;

use duckdb::{
    Appender, CachedStatement, DuckdbConnectionManager, Row, Rows, ToSql,
    types::{FromSql, FromSqlError, FromSqlResult, TimeUnit, ToSqlOutput, ValueRef},
};
use thiserror::Error;

macro_rules! try_result_option {
    ($e:expr) => {
        match $e {
            Err(e) => return Some(Err(e.into())),
            Ok(s) => s,
        }
    };
}

#[derive(Clone)]
pub struct Database {
    pool: r2d2::Pool<DuckdbConnectionManager>,
}

pub struct PooledDatabase {
    conn: r2d2::PooledConnection<DuckdbConnectionManager>,
}

#[derive(Error, Debug)]
pub enum DbError {
    #[error("DuckDB error")]
    DuckDB(#[from] duckdb::Error),
    #[error("r2d2 error")]
    R2D2(#[from] r2d2::Error),
    #[error("Migration {0} does not exist")]
    MigrationDoesntExist(usize),
    #[error("Failed to get DB directory")]
    FailedToGetDBDirectory(#[from] std::io::Error),
}

const MIGRATIONS: [&str; 1] = [include_str!("../migrations/000-initial.sql")];

impl Database {
    pub fn default() -> Result<Self, DbError> {
        let project_dir = directories::ProjectDirs::from("live", "Lockinspiel", "Lockinspiel")
            .ok_or_else(|| {
                DbError::FailedToGetDBDirectory(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Unable to get project directory",
                ))
            })?;
        std::fs::create_dir_all(project_dir.data_dir())
            .map_err(|e| DbError::FailedToGetDBDirectory(e))?;
        Self::new(project_dir.data_dir().join("db.duckdb"))
    }

    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, DbError> {
        let manager = DuckdbConnectionManager::file(path)?;
        let pool = r2d2::Pool::builder().build(manager)?;

        let conn = pool.get()?;
        let migration_table_exists: i32 = conn.query_row(
            "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'migrations'",
            [],
            |row| row.get(0),
        )?;

        if migration_table_exists == 0 {
            conn.execute_batch(
                "CREATE TABLE migrations(version INTEGER); INSERT INTO migrations VALUES (-1);",
            )?;
        }

        let migration_version: i32 =
            conn.query_row("SELECT * FROM migrations", [], |row| row.get(0))?;

        for migration in migration_version + 1..MIGRATIONS.len() as i32 {
            let migration_index = migration as usize;
            let migration_str = MIGRATIONS
                .get(migration_index)
                .ok_or(DbError::MigrationDoesntExist(migration_index))?;
            conn.execute(migration_str, [])?;
            conn.execute("UPDATE migrations SET version = ?", [migration])?;
            tracing::info!(migration, "Applying migration");
        }

        Ok(Database { pool })
    }

    pub fn get(&self) -> Result<PooledDatabase, DbError> {
        Ok(PooledDatabase {
            conn: self.pool.get()?,
        })
    }
}

#[derive(Debug)]
pub struct TimesheetRow {
    pub group: i64,
    pub start_time: JiffTimestamp,
    pub end_time: JiffTimestamp,
    pub activity: i32,
}

impl TryFrom<&Row<'_>> for TimesheetRow {
    type Error = duckdb::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(TimesheetRow {
            group: row.get(0)?,
            start_time: row.get(1)?,
            end_time: row.get(2)?,
            activity: row.get(3)?,
        })
    }
}

impl TimesheetRow {
    #[inline]
    fn as_params(&self) -> [&dyn ToSql; 4] {
        [
            &self.group,
            &self.start_time,
            &self.end_time,
            &self.activity,
        ]
    }
}

pub struct TimesheetTagRow {
    pub timesheet_group: i64,
    pub tag_id: i32,
}

impl TimesheetTagRow {
    #[inline]
    fn as_params(&self) -> [&dyn ToSql; 2] {
        [&self.timesheet_group, &self.tag_id]
    }
}

pub struct TimesheetAppender<'a> {
    appender: Appender<'a>,
}

pub struct TimesheetTagAppender<'a> {
    appender: Appender<'a>,
}

pub struct TimesheetIter<'a> {
    rows: Rows<'a>,
}

pub struct GetTimesheetStmt<'a> {
    stmt: CachedStatement<'a>,
}

impl PooledDatabase {
    pub fn add_to_timesheet(&self, row: TimesheetRow) -> Result<(), DbError> {
        self.conn
            .execute("INSERT INTO timesheet VALUES (?, ?, ?, ?)", row.as_params())?;

        Ok(())
    }

    pub fn stop_timer(&self, now: jiff::Timestamp) -> Result<(), DbError> {
        self.conn.execute("UPDATE timesheet SET end_time = ? WHERE end_time = (SELECT max(end_time) FROM timesheet)", [JiffTimestamp(now)])?;

        Ok(())
    }

    pub fn get_active_timer(&self, now: jiff::Timestamp) -> Result<Option<TimesheetRow>, DbError> {
        let active_timer: Option<TimesheetRow> = match self.conn.query_row(
            "SELECT * FROM timesheet WHERE end_time >= $1",
            [JiffTimestamp(now)],
            |row| TimesheetRow::try_from(row),
        ) {
            Ok(r) => Ok(Some(r)),
            Err(duckdb::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }?;

        Ok(active_timer)
    }

    pub fn add_tag(&self, tag: &str) -> Result<i32, DbError> {
        let tag_id = self.conn.query_row(
            "INSERT INTO tag(tag) VALUES (?) RETURNING id",
            [tag],
            |row| row.get(0),
        )?;
        Ok(tag_id)
    }

    pub fn next_timesheet_group(&self) -> Result<i64, DbError> {
        let next_timesheet = self.conn.query_row(
            "INSERT INTO timesheet_group(timesheet_group) VALUES (DEFAULT) RETURNING timesheet_group",
            [],
            |row| row.get(0),
        )?;
        Ok(next_timesheet)
    }

    pub fn timesheet_appender<'a>(&'a self) -> Result<TimesheetAppender<'a>, DbError> {
        Ok(TimesheetAppender {
            appender: self.conn.appender("timesheet")?,
        })
    }

    pub fn timesheet_tag_appender<'a>(&'a self) -> Result<TimesheetTagAppender<'a>, DbError> {
        Ok(TimesheetTagAppender {
            appender: self.conn.appender("timesheet_tag")?,
        })
    }

    pub fn get_timesheet_stmt<'a>(&'a self) -> Result<GetTimesheetStmt<'a>, DbError> {
        Ok(GetTimesheetStmt {
            stmt: self
                .conn
                .prepare_cached("SELECT * FROM timesheet WHERE start_time >= ? AND end_time < ?")?,
        })
    }
}

impl<'a> GetTimesheetStmt<'a> {
    pub fn get_timesheet(
        &'a mut self,
        start_time: jiff::Timestamp,
        end_time: jiff::Timestamp,
    ) -> Result<TimesheetIter<'a>, DbError> {
        Ok(TimesheetIter {
            rows: self
                .stmt
                .query([JiffTimestamp(start_time), JiffTimestamp(end_time)])?,
        })
    }
}

impl<'a> Iterator for TimesheetIter<'a> {
    type Item = Result<TimesheetRow, DbError>;

    fn next(&mut self) -> Option<Self::Item> {
        let row = try_result_option!(self.rows.next())?;
        let timesheet_row = try_result_option!(TimesheetRow::try_from(row));

        Some(Ok(timesheet_row))
    }
}

impl<'a> TimesheetAppender<'a> {
    #[inline]
    pub fn append_timesheet_row(&mut self, row: TimesheetRow) -> Result<(), DbError> {
        self.appender.append_row(row.as_params())?;
        Ok(())
    }

    #[inline]
    pub fn flush(&mut self) -> Result<(), DbError> {
        self.appender.flush()?;
        Ok(())
    }
}

impl<'a> TimesheetTagAppender<'a> {
    #[inline]
    pub fn append_timesheet_tag_row(&mut self, row: TimesheetTagRow) -> Result<(), DbError> {
        self.appender.append_row(row.as_params())?;
        Ok(())
    }

    #[inline]
    pub fn flush(&mut self) -> Result<(), DbError> {
        self.appender.flush()?;
        Ok(())
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct JiffTimestamp(pub jiff::Timestamp);

/// Taken from here
/// https://docs.rs/duckdb/1.4.1/src/duckdb/types/chrono.rs.html#48
impl ToSql for JiffTimestamp {
    #[inline]
    fn to_sql(&self) -> duckdb::Result<ToSqlOutput<'_>> {
        let date_str = self.0.strftime("%F %T%.f").to_string();
        Ok(ToSqlOutput::from(date_str))
    }
}

/// Taken from here
/// https://docs.rs/duckdb/1.4.1/src/duckdb/types/chrono.rs.html#59
///
/// "YYYY-MM-DD HH:MM:SS"/"YYYY-MM-DD HH:MM:SS.SSS" => ISO 8601 combined date
/// and time without timezone. ("YYYY-MM-DDTHH:MM:SS"/"YYYY-MM-DDTHH:MM:SS.SSS"
/// also supported)
impl FromSql for JiffTimestamp {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Timestamp(tu, t) => {
                let (secs, nsecs) = match tu {
                    TimeUnit::Second => (t, 0),
                    TimeUnit::Millisecond => (t / 1000, (t % 1000) * 1_000_000),
                    TimeUnit::Microsecond => (t / 1_000_000, (t % 1_000_000) * 1000),
                    TimeUnit::Nanosecond => (t / 1_000_000_000, t % 1_000_000_000),
                };
                Ok(JiffTimestamp(
                    jiff::Timestamp::new(secs, nsecs as i32).unwrap(),
                ))
            }
            ValueRef::Date32(d) => Ok(JiffTimestamp(
                jiff::Timestamp::new(24 * 3600 * (d as i64), 0).unwrap(),
            )),
            ValueRef::Time64(TimeUnit::Microsecond, d) => Ok(JiffTimestamp(
                jiff::Timestamp::new(d / 1_000_000, ((d % 1_000_000) * 1_000) as i32).unwrap(),
            )),
            ValueRef::Text(s) => {
                let mut s = std::str::from_utf8(s).unwrap();
                let format = match s.len() {
                    //23:56:04
                    8 => "%T",
                    //2016-02-23
                    10 => "%F",
                    //13:38:47.144
                    12 => "%T%.f",
                    //2016-02-23 23:56:04
                    19 => "%F %T",
                    //2016-02-23 23:56:04.789
                    23 => "%F %T%.f",
                    //2016-02-23 23:56:04.789+00:00
                    29 => "%F %T%.f%:z",
                    _ => {
                        //2016-02-23
                        s = &s[..10];
                        "%F"
                    }
                };
                jiff::Timestamp::strptime(format, s)
                    .map_err(|err| FromSqlError::Other(Box::new(err)))
                    .map(|ts| JiffTimestamp(ts))
            }
            _ => Err(FromSqlError::InvalidType),
        }
    }
}
