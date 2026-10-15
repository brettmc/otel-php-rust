//! Shared database instrumentation support.
//!
//! The framework DB plugins (Laminas, ZF1) each need to turn a driver name or adapter
//! class into a `db.system.name` value. Keeping one mapping here means the same database
//! cannot be reported under two different names depending on which framework the
//! application happens to use.
//!
//! Values are the well-known `db.system.name` values from the semantic conventions
//! registry. The semconv crate does not generate constants for enum *values* (only for
//! attribute keys), so they are spelled out here.
//! <https://opentelemetry.io/docs/specs/semconv/registry/attributes/db/>

/// Fallback for a SQL database we do not recognise.
pub const OTHER_SQL: &str = "other_sql";

const MYSQL: &str = "mysql";
const MARIADB: &str = "mariadb";
const POSTGRESQL: &str = "postgresql";
const SQLITE: &str = "sqlite";
const ORACLE_DB: &str = "oracle.db";
const IBM_DB2: &str = "ibm.db2";
const MICROSOFT_SQL_SERVER: &str = "microsoft.sql_server";
const FIREBIRDSQL: &str = "firebirdsql";

/// Map a PHP driver/extension name (`pdo_mysql`, `mysqli`, `oci8`, ...) to
/// `db.system.name`.
///
/// Accepts the bare PDO driver prefix as used in a DSN (`mysql:host=...` -> `mysql`) as
/// well as the PHP extension spelling, since the frameworks surface both.
pub fn map_driver_name(driver: &str) -> &'static str {
    match driver.to_lowercase().as_str() {
        "mysql" | "mysqli" | "pdo_mysql" => MYSQL,
        "mariadb" | "pdo_mysql_mariadb" => MARIADB,
        "pgsql" | "postgres" | "postgresql" | "pdo_pgsql" => POSTGRESQL,
        "sqlite" | "sqlite3" | "pdo_sqlite" => SQLITE,
        "oci" | "oci8" | "oracle" | "pdo_oci" => ORACLE_DB,
        "ibm" | "db2" | "ibm_db2" | "pdo_ibm" => IBM_DB2,
        "sqlsrv" | "mssql" | "dblib" | "pdo_sqlsrv" | "pdo_dblib" => MICROSOFT_SQL_SERVER,
        "firebird" | "firebirdsql" | "pdo_firebird" => FIREBIRDSQL,
        // ODBC is a connection mechanism, not a DBMS, so the DBMS is unknown here.
        _ => OTHER_SQL,
    }
}

/// Extract the driver prefix from a PDO DSN (`"sqlite:/tmp/db"` -> `"sqlite"`) and map it
/// to `db.system.name`. Returns [`OTHER_SQL`] if the DSN has no `:` separator.
pub fn map_pdo_dsn(dsn: &str) -> &'static str {
    match dsn.find(':') {
        Some(pos) => map_driver_name(&dsn[..pos]),
        None => OTHER_SQL,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_canonical_semconv_values() {
        // these four are Stable in the semconv registry
        assert_eq!(map_driver_name("pdo_mysql"), "mysql");
        assert_eq!(map_driver_name("mariadb"), "mariadb");
        assert_eq!(map_driver_name("pdo_pgsql"), "postgresql");
        assert_eq!(map_driver_name("pdo_sqlsrv"), "microsoft.sql_server");
    }

    #[test]
    fn maps_values_that_the_two_plugins_used_to_disagree_on() {
        // laminas previously reported "oracle"/"db2"/"mssql" while zf1 reported the
        // canonical values; both now agree on the registry spelling
        assert_eq!(map_driver_name("oci8"), "oracle.db");
        assert_eq!(map_driver_name("pdo_oci"), "oracle.db");
        assert_eq!(map_driver_name("ibm_db2"), "ibm.db2");
        assert_eq!(map_driver_name("sqlsrv"), "microsoft.sql_server");
        assert_eq!(map_driver_name("pdo_dblib"), "microsoft.sql_server");
    }

    #[test]
    fn maps_firebird_to_registry_spelling() {
        // the registry value is "firebirdsql", not "firebird"
        assert_eq!(map_driver_name("pdo_firebird"), "firebirdsql");
    }

    #[test]
    fn is_case_insensitive() {
        assert_eq!(map_driver_name("PDO_MySQL"), "mysql");
        assert_eq!(map_driver_name("OCI8"), "oracle.db");
    }

    #[test]
    fn unknown_driver_falls_back() {
        assert_eq!(map_driver_name("not_a_database"), "other_sql");
        assert_eq!(map_driver_name(""), "other_sql");
        // ODBC identifies a connection mechanism, not a DBMS
        assert_eq!(map_driver_name("pdo_odbc"), "other_sql");
    }

    #[test]
    fn parses_pdo_dsn_prefix() {
        assert_eq!(map_pdo_dsn("sqlite:/tmp/test.sqlite"), "sqlite");
        assert_eq!(map_pdo_dsn("mysql:host=localhost;dbname=test"), "mysql");
        assert_eq!(map_pdo_dsn("oci:dbname=//localhost/XE"), "oracle.db");
    }

    #[test]
    fn dsn_without_separator_falls_back() {
        assert_eq!(map_pdo_dsn("sqlite"), "other_sql");
        assert_eq!(map_pdo_dsn(""), "other_sql");
    }
}
